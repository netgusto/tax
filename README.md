# tax, the prompt task list [![CircleCI](https://circleci.com/gh/netgusto/tax.svg?style=svg)](https://circleci.com/gh/netgusto/tax)

Displays / manages the tasks in `~/tasks.md`, or from the file pointed by `$TAX_FILE` if set.

Tasks are markdown:

```
# Things to do

- [x] Do the laundry
- [ ] **Call mom**
- [ ] Send that email
```

## Commands

### tax | tax list

Displays the current list of open tasks.

### tax current

Displays the first open task of the list.

Meant to be included in prompt or tmux status.

### tax cycle

Picks one open task from the list and display it. The picked task changes every minute.

Meant to be included in prompt or tmux status.

### tax view

Alias: `tax cat`.

Displays the content of the taxfile without any processing.

### tax check $TASK_NUM | tax uncheck $TASK_NUM

Checks/Unchecks the task corresponding to the given number `$TASK_NUM`.

### tax focus $TASK_NUM | tax blur $TASK_NUM

Focuses/Blurs the task corresponding to the given number `$TASK_NUM`.

A focused task is a bold task in markdown formatting.

Ex:

```md
- [ ] **This is a focused task**
```

Focused tasks will be displayed with priority over non focused tasks by the `tax current` and `tax cycle` commands.

### tax add "The task"

Aliases: `tax push`, `tax prepend`.

Adds the given task on top of the first task of the list.

### tax append "The task"

Appends the given task after the last task of the list.

### tax prune

Removes all checked tasks from the task list.

### tax edit

Opens the current task file in `$EDITOR`.

### tax which

Tells which tasks file is currently in use. Useful for scripting.

## Include in prompt

```
# Put tax in your $PATH or:
# alias tax=/path/to/tax
```

Then, to display the current task in your bash prompt:
```
export PS1='= $(tax current)
> '
```

Replace `tax current` with `tax cycle` for the displayed task to change every minute.

Ex on my prompt:

![](assets/prompt.png)

## Include in tmux status

In your `tmux.conf`, for instance:

```
set -g status-right '[...your status config...] #(/path/to/tax cycle)'
```

## License

See [LICENSE.md]()
