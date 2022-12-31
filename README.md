# cargo-sandbox
`cargo-sandbox` intends to be a drop-in replacement for `cargo`, with the added benefit
of isolating those commands from other parts of your system - that is, it runs cargo commands
in a "sandbox".

For example, instead of running:
`cargo check`

You can run:
`cargo-sandbox check`

## Threat Model
`cargo-sandbox` intends to protect against a specific attacker with specific goals.

The attacker can execute code on your host but only as part of a cargo command. This can be
accomplished through malicious build scripts, procedural macros, and possibly other methods.

`cargo-sandbox` *does* aim to prevent this attacker from:
1. Reading or writing files outside of the project's appropriate context
2. Gaining access to sensitive environment variables, such as tokens or passwords
3. Gaining access to your crates.io login token

`cargo-sandbox` *does not* aim to prevent this attacker from:
1. Patching your code such that it executes malicious code at runtime

## Implementation
Currently the isolation provided by `cargo-sandbox` is achieved by running the cargo commands
in various docker containers via the docker unix domain socket (currently hardcoded at "/var/run/docker.sock").

Every "project" has its own set of containers. Within a project there are two containers:
1. Build - used for `cargo build`, `cargo check`, `cargo fmt`, etc.
2. Publish - user for `cargo publish`

No data is shared across the Build and Publish containers within a project, no data is shared
across projects at all. In the future, for optimization purposes, there may be some tightly
controlled sharing.

In order to make native dependencies easier to handle the current plan is to leverage `riff`.
See: https://determinate.systems/posts/riff-rust-maintainers


#### Current State
Currently, `cargo-sandbox` is in a very early stage of development. The management of containers is
error prone and somewhat inefficient. Please see the issue tracker for more details, but here's the
quicknotes version based on what I want to get done short term:

- [X] Partial support for `build`, `check`, `publish`
- [ ] Improved handling of native dependencies with `riff`
- [ ] Support for `fmt`, `clippy`
- [ ] Support for `run`, `test`, `bench`
- [ ] Support for custom policies, including more restrictions

### FAQ

#### Does `cargo-sandbox` work on $OS?

Currently `cargo-sandbox` only works on Linux but that is not a fundamental limitation. While the project
is still in its early stages I'd prefer to focus on getting the design right, not on portability.

That said, ensuring a path to broad OS support is a goal of the project.

#### If the attacker can just patch code and execute in production, what's the point?
To briefly address this concern:

1. There are already tons of tools and techniques for protecting arbitrary production programs. While I would
   not call the problem "solved", the ability to constrain a program at runtime is far more mature.
2. Surprisingly, given (1), a dev environment can be even more privileged than a service in production! Consider
   that some developers will have code commit rights, browser session cookies, SSH keys, and other sensitive data.

Indeed I suspect that many attackers are far more comfortable landing on a developer's laptop than in some random
production service that may not even have external networking capabilities. Further, one tool does not need to solve
all problems - in the future `cargo-sandbox` may expand its threat model.

At minimum I hope to (in the future) improve the ability to *audit* for such attacks, even if they are not prevented.

#### I thought that containers weren't security boundaries? Is this adding any value?

You may have heard "containers are not a security boundary". That has not really been true for a long time, especially
if we're talking about Docker containers.

Let's define a security boundary as "a limitation of an attacker's capabilities
that requires an explicit vulnerability to bypass"; that is to say, if I can just "ask" to get out of a sandbox it is *not*
a boundary, but if I need to exploit a vulnerability that will eventually be patched then it *is* a boundary.

So... is a Docker Container a security boundary? It is - trivially so. There is no native means by which a process in a
Docker Container can escape - any such attack would be considered a Docker vulnerability.

Beyond that, Docker containers leverage a number of powerful security features:
1. Namespaces: Docker containers run in isolated namespaces. Namespaces are a feature of the Linux kernel that allow
   processes to have their own view of the system. For example, a process in a Docker container can have its own view of
   the filesystem, network, users, and process tree.

2. Apparmor/SELinux: Docker containers can (and will by default) run with support for Linux's powerful Mandatory Access
   Control (MAC) systems. These systems allow processes to be restricted from accessing certain resources, such as files
   or network ports.

3. Seccomp: Docker containers run with a seccomp profile by default, restricting which system calls processes can make.
   While the default profile may not be ideal, `cargo-sandbox` intends to provide a more restrictive profile with time.

