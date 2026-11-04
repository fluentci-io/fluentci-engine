module CI.Release (build, package, upload, r2) where

import Prelude
import CI.Support (requiredEnv, run, unwrap)
import Effect (Effect)
import FluentCI.Plugin as P

build :: Effect Unit
build = do
  target <- requiredEnv "TARGET"
  run "release-build"
    [ [ "rustup", "target", "add", target ]
    , [ "cargo", "build", "--release", "--locked", "-p", "fluentci-engine", "--target", target ]
    , [ "target/" <> target <> "/release/fluentci-engine", "--version" ]
    ]

package :: Effect Unit
package = do
  target <- requiredEnv "TARGET"
  version <- requiredEnv "RELEASE_VERSION"
  let archive = "fluentci-engine_" <> version <> "_" <> target <> ".tar.gz"
  run "release-package"
    [ [ "env", "COPYFILE_DISABLE=1", "tar", "-czf", archive, "-C", "target/" <> target <> "/release", "fluentci-engine" ]
    , [ "tar", "-tzf", archive ]
    ]
  P.pipeline "release-checksum"
    # P.withEnv "ARCHIVE" archive
    # P.shell """
shasum -a 256 "$ARCHIVE" > "$ARCHIVE.sha256"
"""
    # P.exec [ "printf", "%s\n", "Release packaged" ]
    # P.stdout >>= unwrap >>= P.output >>= unwrap

upload :: Effect Unit
upload = do
  target <- requiredEnv "TARGET"
  version <- requiredEnv "RELEASE_VERSION"
  let archive = "fluentci-engine_" <> version <> "_" <> target <> ".tar.gz"
  run "release-upload" [ [ "gh", "release", "upload", version, archive, archive <> ".sha256", "--clobber" ] ]

r2 :: Effect Unit
r2 = do
  target <- requiredEnv "TARGET"
  version <- requiredEnv "RELEASE_VERSION"
  account <- requiredEnv "ACCOUNT_ID"
  let archive = "fluentci-engine_" <> version <> "_" <> target <> ".tar.gz"
  let uploadFile filename = [ "aws", "s3", "cp", filename, "s3://assets/fluentci-engine/" <> version <> "/" <> filename, "--endpoint-url", "https://" <> account <> ".r2.cloudflarestorage.com", "--only-show-errors" ]
  run "release-r2" [ uploadFile archive, uploadFile (archive <> ".sha256") ]
