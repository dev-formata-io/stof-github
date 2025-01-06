//
// Copyright 2024 Formata, Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::{collections::HashMap, time::Duration};
use anyhow::Result;
use stof::{Format, SDoc};
use ureq::{Agent, AgentBuilder};


/// Stof GitHub Format.
pub struct GitHubFormat {
    /// Format Repo ID.
    /// Ex. "formata" means format is "github:formata".
    pub repo_id: String,

    /// Repository owner.
    pub owner: String,

    /// Repository name.
    pub repo: String,

    /// Headers.
    pub headers: HashMap<String, String>,

    /// Agent.
    pub agent: Agent,
}
impl GitHubFormat {
    /// Create a new GitHub Format.
    pub fn new(repo: &str, owner: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Accept".to_string(), "application/vnd.github.raw+json".to_string());
        headers.insert("X-GitHub-Api-Version".to_string(), "2022-11-28".to_string());
        Self {
            repo_id: repo.to_owned(),
            owner: owner.to_owned(),
            repo: repo.to_owned(),
            headers,
            agent: AgentBuilder::new()
                .timeout_read(Duration::from_secs(3))
                .timeout_write(Duration::from_secs(3))
                .build(),
        }
    }

    /// The URL for a request into this GitHub repository.
    pub fn url(&self, path: &str) -> String {
        format!("https://api.github.com/repos/{}/{}/contents/{}", self.owner, self.repo, path)
    }

    /// Get the string contents for a file path into this GitHub repository.
    pub fn get(&self, file_path: &str) -> Result<String> {
        let url = self.url(file_path);
        let mut request = self.agent.get(&url);
        for (key, value) in &self.headers {
            request = request.set(key, value);
        }
        let response = request.call()?;
        Ok(response.into_string()?)
    }
}
impl Format for GitHubFormat {
    /// How this format will be accessed in Stof.
    /// For example, if repo_id is "formata", using this library would be the format identifier "github:formata".
    ///
    /// `import github:formata "myfile.stof" as Import;`
    fn format(&self) -> String {
        format!("github:{}", self.repo_id)
    }

    /// The GitHub format only allows a file import.
    /// Gets the contents of the file at a path in this GitHub repo, then imports it as a string using the file extension.
    /// Will error if a Format with the requested file extension is not available in the doc.
    fn file_import(&self, pid: &str, doc: &mut SDoc, _format: &str, full_path: &str, extension: &str, as_name: &str) -> Result<()> {
        let contents = self.get(full_path)?;
        doc.string_import(pid, extension, &contents, as_name)
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use stof::SDoc;
    use crate::GitHubFormat;

    #[test]
    fn test() {
        let mut doc = SDoc::default();
        doc.load_format(Arc::new(GitHubFormat::new("stof", "dev-formata-io"))); // github:stof

        doc.string_import("main", "stof", r#"
            
            import github:stof "web/deno.json"; // Will import deno.json using the "json" format into "root"

            #[test('@formata/stof')]
            fn name(): str {
                return self.name;
            }

            #[test('Apache-2.0')]
            fn license(): str {
                return self.license;
            }

        "#, "").unwrap();

        let res = doc.run_tests(true, None);
        match res {
            Ok(message) => {
                println!("{message}");
            },
            Err(error) => {
                panic!("{error}");
            }
        }
    }
}
