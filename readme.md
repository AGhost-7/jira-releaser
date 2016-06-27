# Jira Releaser
A tool which diffs commit logs in two branches, finding all valid Jira
issue identifiers to create a releases page automatically for you.

## How to Use
Well...

```
Jira Release Tool 0.1.0
Jonathan Boudreau

USAGE:
    jira-releaser [FLAGS] --release-branch <Release branch> --latest-branch <Latest branch> --url <Jira URL> --project-id <Project Id> --version-name <Version name> --username <Username> --password <Password>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -U, --url <Jira URL>                     This is the api root url for your Jira project.
    -l, --latest-branch <Latest branch>      The branch which is going to be merged to trigger the release [default: develop] 
    -p, --password <Password>                Jira password. Falls back to JIRA_PASSWORD environment variable
    -P, --project-id <Project Id>            Project id or key on Jira
    -r, --release-branch <Release branch>    The branch which once the release is created, will be merged into [default: master] 
    -u, --username <Username>                Your Jira username. Falls back to the JIRA_USERNAME environment variable
    -v, --version-name <Version name>        The version name to use for the release.
```

## Authentication
Currently uses Basic Auth over HTTPS thanks to OpenSSL. N.b., it is
recommended to use environment variables instead of passing it as an
argument directly to the commit line tool for the credentials. You
should place the credentials in a separate file and source it from your
`.bashrc` or `.bash_profile`. Doing this, you can place the credentials
in an ecryptfs folder, or something similar.
