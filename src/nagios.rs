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

pub struct NagiosService<'a> {
    name: &'a str
}

impl<'a> NagiosService<'a> {
    pub fn new(name: &str) -> NagiosService {
        NagiosService {
            name: name
        }
    }

    pub fn report(&self, status: NagiosStatus) {
        let (status_code, status_string, description) = match status {
            NagiosStatus::Ok(ref description) => (0, "OK", description),
            NagiosStatus::Warning(ref description) => (1, "WARNING", description),
            NagiosStatus::Critical(ref description) => (2, "CRITICAL", description),
            NagiosStatus::Unknown(ref description) => (3, "UNKNOWN", description)
        };
        println!("{} {}: {}", self.name, status_string, description);
        ::std::process::exit(status_code);
    }
}

pub enum NagiosStatus {
    Ok(String),
    Warning(String),
    Critical(String),
    Unknown(String)
}
