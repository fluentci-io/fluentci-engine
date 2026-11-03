/** Native-plan matrices. Values stay pure until run is called inside Wasm. */
type matrixCase<'a>
type task
type plan
type taskResult = {id: string, status: string, stdout: string, stderr: string}
type runResult = {outcome: string, tasks: array<taskResult>}

@module("./matrix_ffi.mjs") external named: (string, 'a) => matrixCase<'a> = "named"
@module("./matrix_ffi.mjs") external axis: (string, array<string>) => array<matrixCase<string>> = "axis"
@module("./matrix_ffi.mjs") external includeCases: (array<matrixCase<'a>>, array<matrixCase<'a>>) => array<matrixCase<'a>> = "include"
@module("./matrix_ffi.mjs") external exclude: (array<matrixCase<'a>>, 'a => bool) => array<matrixCase<'a>> = "exclude"
@module("./matrix_ffi.mjs") external task: (string, string, array<string>, string) => task = "task"
@module("./matrix_ffi.mjs") external withEnv: (task, string, string) => task = "withEnv"
@module("./matrix_ffi.mjs") external cached: (task, array<string>, array<string>, string) => task = "cached"
@module("./matrix_ffi.mjs") external afterJobs: (task, array<task>) => task = "after"
@module("./matrix_ffi.mjs") external parallel: (array<task>, int) => plan = "parallel"
@module("./matrix_ffi.mjs") external withCacheDirectory: (plan, string) => plan = "withCacheDirectory"
@module("./matrix_ffi.mjs") external withFailFast: (plan, bool) => plan = "withFailFast"
@module("./matrix_ffi.mjs") external compile: plan => string = "compile"
@module("./matrix_ffi.mjs") external productImpl: (array<matrixCase<'a>>, array<matrixCase<'b>>, array<matrixCase<('a, 'b)>> => result<array<matrixCase<('a, 'b)>>, string>, string => result<array<matrixCase<('a, 'b)>>, string>) => result<array<matrixCase<('a, 'b)>>, string> = "product"
@module("./matrix_ffi.mjs") external matrixImpl: (string, array<matrixCase<'a>>, 'a => task, array<task> => result<array<task>, string>, string => result<array<task>, string>) => result<array<task>, string> = "matrix"
@module("./matrix_ffi.mjs") external runImpl: (plan, string, runResult => result<runResult, string>, string => result<runResult, string>) => result<runResult, string> = "run"

let product = (left, right) => productImpl(left, right, value => Ok(value), error => Error(error))
let matrix = (name, cases, build) => matrixImpl(name, cases, build, value => Ok(value), error => Error(error))
let run = (plan, directory) => runImpl(plan, directory, value => Ok(value), error => Error(error))
