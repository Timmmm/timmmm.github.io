# Recursive wildcard from https://stackoverflow.com/a/18258352/265521
rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))

# Find all markdown files recursively.
inputs := $(call rwildcard,.,*.md)

# Convert list from foo/bar.md to foo/bar.html
outputs := $(patsubst %.md,%.html,$(inputs))

.PHONY: all

all: $(outputs)

%.html: %.md
	pandoc -s --css=../style.css $< -o $@
