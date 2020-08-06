# Sections

Support for Sections within a single task file

:* a section is a markdown header
:* 1 section => equivalent to no section
:* display sections in list
:* display section of task in current/cycle
:* ability to focus section
:  * when focus section, blur other focused sections
:* current / cycle: pick only from focused task
:* add, append: default top 1st section, bottom last section
* add task --section Section => Add Task to section
  * Add to section with better match
  * Or Index sections by letter; add a "Something"
  * added tasks are always added 2 lines below the last text line of the section, except if there's already a task in the section, in which case the position of this task will be the place of insertion
* when add without --section, add to focused section if any
* move task to section
:* when focused, display only focused section in list (+hint "Other tasks in hidden sections...")

# Scriptability

* add export command for json export of model
  * Or even better, add --json to every command to make their output JSON
* add optional --file, similar to TASK_FILE env variable

# Web UI

* Maybe (prolly not) add web UI ?

# Log file

* Integrate feature? Or let it rely on outside scripting?