# tax, the prompt task list

Displays the next pending task from `~/taxfile`.

Tasks are markdown:

```
# Task list

- [*] Do the laundry
- [ ] Be good
```

## Include in prompt

```
alias __tax=/path/to/tax

export PS1='= $(__tax)
> '
```
