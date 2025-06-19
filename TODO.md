# To-do list

- [ ] Refactor commands to not be single functions spanning one hundred lines.
- [ ] Use tracing module to emit progress updates to the user.
- [ ] Add proper documentation everywhere.
- [ ] Add proper tests.
- [ ] Move some stuff to `axiom_core` (consider the inverse of moving
  everything back into one crate if all dependencies are just single files)
- [ ] Cache the results coming from the PaperMC API in some way. I want to be
  able to do most work offline, only needing to rely on the PaperMC API if
  running the `update` command.
- [ ] I think I should be using `assert!` or `debug_assert!` more often.
