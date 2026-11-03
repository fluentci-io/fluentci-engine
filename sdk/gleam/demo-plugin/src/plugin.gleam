import fluentci/plugin

pub fn greet(input: String) -> Result(String, String) {
  plugin.pipeline("gleam-greeting")
  |> plugin.exec("/usr/bin/printf", ["%s", input])
  |> plugin.stdout
}
