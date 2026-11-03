module Sccache where

import Prelude
import Data.Array as Array
import Data.Either (Either(..))
import Data.Maybe (Maybe(..))
import Data.String as String
import Data.String.CodeUnits as CodeUnits
import Data.String.Pattern (Pattern(..))
import Effect (Effect)
import Effect.Exception (throw)
import FluentCI.Host.Client as Report
import FluentCI.Host.Plan as Plan
import FluentCI.Plugin as P

type Artifact = { target :: String, checksum :: String }
type Config = { project :: String, run :: String, install :: String, cache :: String, path :: String }

-- Platform selection and checksums are data, independent of the effect boundary.
artifactFor :: String -> String -> Either String Artifact
artifactFor os arch = case os, arch of
  "linux", "x86_64" -> Right { target: "x86_64-unknown-linux-musl", checksum: "45f1447fbe231e3037bde351ef70677dd212216c8d62ae7ca409fecc4d6acc89" }
  "linux", "aarch64" -> Right { target: "aarch64-unknown-linux-musl", checksum: "2b3284d5da3b46a47dc4229e75bb7b88ac4aa99c8d754fb7d2f84997e5a4354a" }
  "macos", "x86_64" -> Right { target: "x86_64-apple-darwin", checksum: "1dade83cc49eeb42337565eccd534b05982820a8e44b851bd7937467a18c7aef" }
  "macos", "aarch64" -> Right { target: "aarch64-apple-darwin", checksum: "308184519b646f5125289e8515b36f6ca65a13a041923994aebe702348674e8e" }
  _, _ -> Left ("Unsupported sccache provisioning platform: " <> os <> "/" <> arch)

validateArguments :: Array String -> Either String { project :: String, run :: String, cargo :: Array String }
validateArguments arguments = case Array.uncons arguments of
  Just { head: project, tail } -> case Array.uncons tail of
    Just { head: run, tail: cargo }
      | project /= "" && run /= "" && not (Array.any overrideProfile cargo) -> Right { project, run, cargo }
    _ -> invalid
  _ -> invalid
  where
  overrideProfile value = value == "--profile" || String.contains (Pattern "--profile=") value
  invalid = Left "Expected [project_directory, fresh_run_directory, ...cargo_build_arguments]; profile overrides are not supported"

require :: forall a. P.Result a -> Effect a
require = case _ of
  Left error -> throw (P.errorMessage error)
  Right value -> pure value

getEnv :: String -> Effect String
getEnv name = P.getEnv name >>= require

absoluteDirectory :: String -> String -> Effect String
absoluteDirectory name suffix = do
  configured <- getEnv name
  home <- getEnv "HOME"
  let directory = if configured == "" then home <> "/" <> suffix else configured
  if CodeUnits.take 1 directory == "/" && (configured /= "" || home /= "") then pure directory
  else throw ("Set HOME or an absolute " <> name)

-- All programs receive literal argv. No shell interprets user paths or arguments.
task :: Config -> String -> String -> Array String -> Plan.Task
task config id program args = Plan.task id program config.project
  # Plan.withArgs args
  # Plan.withEnv "PATH" (config.install <> ":" <> config.path)
  # Plan.withTimeout 180000

nativePlan :: Array Plan.Task -> Plan.Plan
nativePlan tasks = Plan.plan tasks # Plan.withInheritedEnv true # Plan.withJobs 1

execute :: String -> Plan.Plan -> Effect Report.RunResult
execute directory plan = P.runPlan directory plan >>= require

ensureSucceeded :: String -> Report.RunResult -> Effect Report.RunResult
ensureSucceeded directory report =
  if report.outcome == Just Report.RunSucceeded then pure report
  else throw ("sccache execution failed; see " <> directory <> "/result.json")

runTasks :: String -> Array Plan.Task -> Effect Report.RunResult
runTasks directory tasks = execute directory (nativePlan tasks) >>= ensureSucceeded directory

findTask :: String -> Report.RunResult -> Effect Report.TaskResult
findTask id report = case Array.find (\value -> value.id == id) report.tasks of
  Just value -> pure value
  Nothing -> throw ("Missing task result: " <> id)

readText :: String -> Effect String
readText path = P.pipeline "sccache-read-result"
  # P.exec [ "/bin/cat", path ]
  # P.stdout
  # (_ >>= require)

archiveName :: Artifact -> String
archiveName artifact = "sccache-v0.18.0-" <> artifact.target

downloadPlan :: Config -> Artifact -> String -> Plan.Plan
downloadPlan config artifact stage = nativePlan
  [ task config "download" "/usr/bin/curl"
      [ "--fail"
      , "--silent"
      , "--show-error"
      , "--location"
      , "--retry"
      , "3"
      , "https://github.com/mozilla/sccache/releases/download/v0.18.0/" <> archiveName artifact <> ".tar.gz"
      , "--output"
      , stage <> "/archive.tar.gz"
      ]
  ]

installPlan :: Config -> Artifact -> String -> Plan.Plan
installPlan config artifact stage = nativePlan [ extract, executable, move ]
  where
  binary = stage <> "/" <> archiveName artifact <> "/sccache"
  extract = task config "extract" "/usr/bin/tar"
    [ "-xzf", stage <> "/archive.tar.gz", "-C", stage, archiveName artifact <> "/sccache" ]
  executable = task config "executable" "/bin/chmod" [ "755", binary ] # Plan.after [ extract ]
  move = task config "install" "/bin/mv" [ binary, config.install <> "/sccache" ] # Plan.after [ executable ]

-- Cleanup is attempted on both checksum mismatch and installation failure.
provision :: Config -> Effect String
provision config = do
  os <- P.getOS >>= require
  arch <- P.getArch >>= require
  artifact <- case artifactFor os arch of
    Left message -> throw message
    Right value -> pure value
  let
    mkdir = task config "directory" "/bin/mkdir" [ "-p", config.install ]
    temporary = task config "temporary" "/usr/bin/mktemp" [ "-d", config.install <> "/.sccache-XXXXXXXX" ] # Plan.after [ mkdir ]
  staging <- runTasks (config.run <> ".stage") [ mkdir, temporary ]
  temporaryResult <- findTask "temporary" staging
  stage <- String.trim <$> readText temporaryResult.stdout
  download <- P.runPlan (config.run <> ".download") (downloadPlan config artifact stage)
  hash <- case download of
    Right report | report.outcome == Just Report.RunSucceeded -> P.sha256 (P.file (stage <> "/archive.tar.gz"))
    _ -> pure (Right "")
  installed <- case hash of
    Right actual | String.trim actual == artifact.checksum -> P.runPlan (config.run <> ".install") (installPlan config artifact stage)
    _ -> pure download
  void $ P.runPlan (config.run <> ".stage-cleanup") (nativePlan [ task config "cleanup" "/bin/rm" [ "-rf", stage ] ])
  case hash of
    Right actual | String.trim actual == artifact.checksum -> do
      report <- require installed
      void $ ensureSucceeded (config.run <> ".install") report
      pure (config.install <> "/sccache")
    _ -> throw "sccache download or SHA-256 verification failed; installation stopped"

buildPlan :: Config -> String -> String -> Array String -> Plan.Plan
buildPlan config binary socket arguments = nativePlan [ server, compile, stats ]
  # Plan.withMaxServices 1
  where
  environment value = value
    # Plan.withEnv "SCCACHE_DIR" config.cache
    # Plan.withEnv "SCCACHE_CACHE_SIZE" "1G"
    # Plan.withEnv "SCCACHE_SERVER_UDS" socket
    # Plan.withEnv "CARGO_INCREMENTAL" "0"
    # Plan.withEnv "RUSTC_WRAPPER" binary
  readiness = { program: "/bin/test", args: [ "-S", socket ], interval_ms: 100, timeout_ms: 1000 }
  server = task config "compiler-cache" binary []
    # environment
    # Plan.withEnv "SCCACHE_START_SERVER" "1"
    # Plan.withEnv "SCCACHE_NO_DAEMON" "1"
    # Plan.withTimeout 1830000
    # Plan.withResources [ "sccache-storage:" <> config.cache ]
    # Plan.asService ((Plan.service readiness) { startup_timeout_ms = 15000 })
  compile = task config "build" "cargo" ([ "build", "--release", "--locked" ] <> arguments)
    # environment
    # Plan.withTimeout 1800000
    # Plan.after [ server ]
  stats = task config "stats" binary [ "--show-stats", "--stats-format=json" ]
    # environment
    # Plan.after [ server, compile ]

buildProject :: Array String -> Effect String
buildProject input = do
  arguments <- case validateArguments input of
    Left message -> throw message
    Right value -> pure value
  install <- absoluteDirectory "SCCACHE_INSTALL_DIR" ".local/bin"
  cache <- absoluteDirectory "SCCACHE_DIR" ".cache/sccache"
  path <- getEnv "PATH"
  let
    config = { project: arguments.project, run: arguments.run, install, cache, path }
    guard = Plan.task "fresh-run" "/bin/test" "." # Plan.withArgs [ "!", "-e", config.run ]
    identity = task config "identity" "/usr/bin/printf" [ "%s", config.run ] # Plan.after [ guard ]
    locate = task config "locate" "/usr/bin/which" [ "sccache" ] # Plan.after [ identity ]
  probe <- execute (config.run <> ".probe") (nativePlan [ guard, identity, locate ])
  guardResult <- findTask "fresh-run" probe
  unless (guardResult.status == Report.TaskSucceeded) (throw "The run directory must be fresh")
  identityResult <- findTask "identity" probe
  digest <- P.sha256 (P.file identityResult.stdout) >>= require
  let socket = "/tmp/fluentci-sccache-" <> CodeUnits.take 24 (String.trim digest) <> ".sock"
  located <- findTask "locate" probe
  binary <-
    if located.status == Report.TaskSucceeded then String.trim <$> readText located.stdout
    else if located.exit_code == Just 1 then provision config
    else throw "Could not discover sccache on PATH; see the probe logs"
  result <- P.runPlan config.run (buildPlan config binary socket arguments.cargo)
  -- Direct foreground services need no wrapper process. The executor reaps them.
  -- Remove the filesystem socket after either a successful or failed build.
  void $ P.runPlan (config.run <> ".socket-cleanup") (nativePlan [ task config "remove-socket" "/bin/rm" [ "-f", socket ] ])
  report <- require result
  void $ ensureSucceeded config.run report
  readText (config.run <> "/result.json")
