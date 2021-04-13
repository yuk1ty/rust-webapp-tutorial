# rust-webapp-tutorial

An introduction to build web application in Rust step by step.

## How this works?

This tutorial uses `cargo workspace` -- is a feature in cargo. You can run each project by `cargo run -p [project_name]`. For example:

- `health-check`: `cargo run -p health-check`
- `first-todo-list`: `cargo run -p first-todo-list`

## Contents

The tutorial has x contents below:

- `health-check`: At the beginning, we try to implement a simple endpoint. Create the endpoint alaways returns status code `200 OK` and its body (`"OK"`). First touch for `async/.await`, asynchronous runtimes and other advanced Rust's grammer.
- `first-todo-list`: Working on JSON response and introducing some convenient crates. Throughout this section, you will be able to create an endpoint to return JSON response. In addition, we introduce some crates (generating uuid, handling datetime) for the sake of real-world applications.

## Requirements

- Rust >= 1.50.0
