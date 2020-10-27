#!/bin/bash

for FILE in *.ipe
do
  # Get the number of pages
  for PAGE in $(seq $(grep "<page>" "${FILE}" | wc -l))
  do
    /Applications/Ipe.app/Contents/MacOS/iperender -svg -page $PAGE "${FILE}" "${FILE%.ipe}_p${PAGE}.svg"
  done
done
