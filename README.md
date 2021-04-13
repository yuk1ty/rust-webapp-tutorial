# rust-webapp-tutorial

An introduction to build web application in Rust step by step.

## How this works?

This tutorial uses `cargo workspace` -- is a feature in cargo. You can run each project by `cargo run -p [project_name]`. For example:

- `health-check`: `cargo run -p health-check`

## Contents

The tutorial has x contents below:

- `health-check`: At the beginning, we try to implement a simple endpoint. Create the endpoint alaways returns status code `200 OK` and its body (`"OK"`). First touch for `async/.await`, asynchronous runtimes and other advanced Rust's grammer.
