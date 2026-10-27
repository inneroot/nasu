<div>
  <div align="center" style="display: block; text-align: center;">
    <img src="https://raw.githubusercontent.com/EstebanBorai/nasu/main/docs/nurse-emoji.png" height="120" width="120" />
  </div>
  <h1 align="center">ナース (Nāsu)</h1>
  <h4 align="center">
    🧑🏻‍⚕️ Command-line utility which poll on remote addresses in order to perform status checks periodically
  </h4>
</div>

<div align="center">

  [![Crates.io](https://img.shields.io/crates/v/nasu.svg)](https://crates.io/crates/nasu)
  [![Documentation](https://docs.rs/nasu/badge.svg)](https://docs.rs/nasu)
  ![Build](https://github.com/EstebanBorai/nasu/workflows/build/badge.svg)
  ![Lint](https://github.com/EstebanBorai/nasu/workflows/clippy/fmt/badge.svg)
  ![Tests](https://github.com/EstebanBorai/nasu/workflows/tests/badge.svg)

</div>

## Motivation

Nāsu (from Japanese ナース [Nāsu], which means nurse), is a command-line utility
to perform checks on remote addresses periodically.

## Usage

> This is a fork of the original [EstebanBorai/nasu](https://github.com/EstebanBorai/nasu).
> It adds colored output (`--color`), a per-request timeout, concurrency guards,
> and a reworked timestamp format. All task configuration is HTTP-only.

Build and install from this repository:

```shell
git clone git@github.com:inneroot/nasu.git
cd nasu
cargo install --path .
```

Create a `nasu.json` file and execute `nasu`.

Nasu will execute the tasks specified on `nasu.json` and will provide details
in your terminal as follows:

```
Log Time                       | Task            | HTTP. Status Code    | Req. Time
=====================================================================================
2026-10-27 17:42:01 UTC+03:00  | httpbin post    | 200                  | 252ms
2026-10-27 17:42:02 UTC+03:00  | httpbin get     | 200                  | 1284ms
2026-10-27 17:52:01 UTC+03:00  | httpbin post    | 200                  | 240ms
2026-10-27 17:52:02 UTC+03:00  | httpbin get     | 200                  | 1270ms
2026-10-27 18:02:01 UTC+03:00  | httpbin post    | 200                  | 235ms
```

With the `--color` flag the `HTTP. Status Code` and `Req. Time` cells are
colored (green for `200`/fast, yellow for `3xx`/`4xx`/medium, red for `5xx`/slow
or connection errors).

## Command-line options

Nasu accepts the following command-line flags:

Flag | Description
--- | ---
`--color` | Colorizes the output. Without the flag the output contains no ANSI escape codes (suitable for piping or logging).

When `--color` is enabled, the `HTTP. Status Code` and `Req. Time` columns are
colored based on the outcome:

Status | Color
--- | ---
HTTP `200` | Green
HTTP `3xx` / `4xx` | Yellow
HTTP `5xx` | Red
Connection errors (e.g. `CONNECTION REFUSED`, `TIMEOUT`) | Red

Condition (request delay) | Color
--- | ---
`< 500ms` | Green
`500ms` – `999ms` | Yellow
`>= 1000ms` | Red

Example:

```shell
nasu --color
```

## Terminology

The following terminology is used to refer to nasu main components:

### Service

Service to perform task against through the Worker, a service is created
from a task defined in the `nasu.json` file.

### Worker

Worker in responsible of performing the task, holds the logic to interact
with the service in question.

### Task

Defintion of steps to be performed by the worker. Is provided
in the `nasu.json` file.

### `nasu.json` Reference

`nasu.json` is the default configuration file for Nasu. This file is
parsed at startup by Nasu to initialize `Workers`.

The `nasu.json` file is composed by an array of `Task` objects as shown
below:

```json
// nasu.json
[
  {
    "id": "HTTPBIN POST Request",
    "task": {
      "interval": "* */10 * * * *"
    },
    "params": {
      "url": "http://httpbin.org/post",
      "method": "POST",
      "headers": {
        "authorization": "Bearer <Token>",
        "content-type": "application/json"
      }
    }
  }
]
```

Each of these `Task` must contain the following properties:

Property | Description | Required | Possible Values
--- | --- | --- | ---
`id` | The id of the service. Used as reference for the user | Yes | N/A
`task` | Task configuration | Yes | N/A
`task.interval` | Cron defintion to specify when to perform the test | Yes | N/A
`params` | Params for the `Worker` used on perform. [Refer to Worker Params](#worker-params) | Yes | N/A

### Task `interval` field

sec  min   hour   day of month   month   day of week   year

The `interval` field on a task uses a [cron](https://en.wikipedia.org/wiki/Cron) definition.

```
 ┌────────────── second (0 - 59)
 │ ┌───────────── minute (0 - 59)
 │ │ ┌───────────── hour (0 - 23)
 │ │ │ ┌───────────── day of the month (1, 15)
 │ │ │ │ ┌───────────── month (Mon, Wed, Fri)
 │ │ │ │ │ ┌───────────── day of the week (Mon,Wed,Fri)
 │ │ │ │ │ │ ┌──────────── year
 │ │ │ │ │ │ │
 │ │ │ │ │ │ │
 * * * * * * *
```

An example:

```
0 30 9,12,15 1,15 May-Aug Mon,Wed,Fri 2018/2
```

This application uses [cron](https://docs.rs/cron) crate internally
to parse and calculate time intervals.

### Worker Params

`Worker` params may vary based on the `type` of worker in question.
Properties defined below belong to an object specified in the `Task`
object, inside of the `params` property.

#### HTTP Worker Params

- Type: `http`
- Property: `params`

Property | Description | Required | Possible Values
--- | --- | --- | ---
`url` | URL to perform the HTTP Request | Yes | N/A
`method` | HTTP Method to perform the request with | Yes | `GET`, `PATCH`, `POST`, `PUT`, `DELETE`
`headers` | HTTP Headers to provide to the request | No | N/A

## Release

In order to create a release you must push a Git tag as follows

```shell
git tag -a <version> -m <message>
```

**Example**

```shell
git tag -a  v0.1.0 -m "First release"
```

> Tags must follow SemVer conventions and must be prefixed with a lowercase `v` letter.

Then push tags as follows:

```shell
git push origin main --follow-tags
```
