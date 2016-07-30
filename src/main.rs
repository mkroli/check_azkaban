/*
 * Copyright 2016 Michael Krolikowski
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

extern crate azkaban_client;
extern crate clap;

mod nagios;

use azkaban_client::Azkaban;
use azkaban_client::response::Execution;
use azkaban_client::error::AzkabanError;
use clap::{Arg, App};
use nagios::{NagiosService, NagiosStatus};
use std::fmt;

enum CheckAzkabanError {
    AzkabanError(AzkabanError),
    NoExecutionFoundError
}

impl From<AzkabanError> for CheckAzkabanError {
    fn from(err: AzkabanError) -> CheckAzkabanError {
        CheckAzkabanError::AzkabanError(err)
    }
}

impl fmt::Display for CheckAzkabanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &CheckAzkabanError::AzkabanError(ref err) => err.fmt(f),
            &CheckAzkabanError::NoExecutionFoundError => write!(f, "No completed execution found")
        }
    }
}

fn check_azkaban(base_url: &str, user: &str, password: &str, project: &str, flow: &str) -> Result<Execution, CheckAzkabanError> {
    let azkaban = try!(Azkaban::authenticated(base_url, user, password));
    for i in 0.. {
        let mut executions: Vec<Execution> = try!(azkaban.executions(project, flow, i, 1)).executions;
        if executions.is_empty() {
            break;
        } else {
            let execution = executions.remove(0);
            if execution.status != "READY" && execution.status != "PREPARING" && execution.status != "RUNNING" && execution.status != "PAUSED" {
                return Ok(execution)
            }
        }
    }
    Err(CheckAzkabanError::NoExecutionFoundError)
}

fn format_duration(millis: u64) -> String {
    let seconds = (millis / 1000) % 60;
    let minutes = (millis / 1000 / 60) % 60;
    let hours = millis / 1000 / 60 / 60;
    let millis = millis % 1000;
    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
}

#[test]
fn test_format_duration() {
    assert_eq!("00:00:00.000", format_duration(0));
    assert_eq!("12:34:56.789", format_duration(45296789));
    assert_eq!("123:45:43.210", format_duration(445543210));
}

fn description(execution: &Execution) -> String {
    format!("{}, took {}", execution.status, format_duration((execution.end_time - execution.start_time) as u64))
}

fn main() {
    let matches = App::new("check_azkaban")
        .version("0.1")
        .author("Michael Krolikowski")
        .about("Nagios check plugin for Azkaban")
        .arg(Arg::with_name("base_url")
                 .short("b")
                 .long("base-url")
                 .value_name("url")
                 .help("The Base-URL of Azkaban")
                 .required(true))
        .arg(Arg::with_name("username")
                 .short("u")
                 .long("username")
                 .value_name("username")
                 .required(true))
        .arg(Arg::with_name("password")
                 .short("p")
                 .long("password")
                 .value_name("password")
                 .required(true))
        .arg(Arg::with_name("project")
                 .long("project")
                 .value_name("project")
                 .required(true))
        .arg(Arg::with_name("flow")
                 .long("flow")
                 .value_name("flow")
                 .required(true))
        .get_matches();

    let base_url = matches.value_of("base_url").unwrap();
    let username = matches.value_of("username").unwrap();
    let password = matches.value_of("password").unwrap();
    let project = matches.value_of("project").unwrap();
    let flow = matches.value_of("flow").unwrap();

    let execution = check_azkaban(base_url, username, password, project, flow);

    let service = NagiosService::new("Azkaban");
    let status = match execution {
        Ok(ref execution) => {
            if execution.status == "SUCCEEDED" {
                NagiosStatus::Ok(description(execution))
            } else {
                NagiosStatus::Critical(description(execution))
            }
        },
        Err(err) => NagiosStatus::Unknown(format!("{}", err))
    };
    service.report(status);
}
