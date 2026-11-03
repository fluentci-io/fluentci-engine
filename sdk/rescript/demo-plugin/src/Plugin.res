@val @scope("Host") external input: unit => string = "inputString"
@val @scope("Host") external output: string => unit = "outputString"

let greet = () => {
  let result = Guest.pipeline("rescript-greeting")
    ->Guest.exec(["/usr/bin/printf", "%s", input()])
    ->Guest.stdout
  switch result {
  | Ok(value) => output(value)
  | Error(message) => JsError.throwWithMessage(message)
  }
}
