

The seccomp profiles are based on the default profile:

https://raw.githubusercontent.com/moby/moby/master/profiles/seccomp/default.json

### Notable changes

* Removal of io_uring system calls
    * io_setup
    * io_submit
    * io_uring_enter
    * io_uring_register
    * io_uring_setup

#### Build Specific



#### Publish Specific
