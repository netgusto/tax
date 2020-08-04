# Sections

Support for Sections within a single task file

:* a section is a markdown header
:* 1 section => equivalent to no section
:* display sections in list
:* display section of task in current/cycle
* ability to focus section
  * when focus section, blur other focused sections
* current / cycle: pick only from focused task
:* add, append: default top 1st section, bottom last section
* add task --section Section => Add Task to section
  * Add to section with better match
  * Or Index sections by letter; add a "Something"
  * added tasks are always added 2 lines below the last text line of the section, except if there's already a task in the section, in which case the position of this task will be the place of insertion
* tax section add "Section" (really?)