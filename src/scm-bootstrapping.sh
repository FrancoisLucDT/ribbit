#!/bin/bash

# For measuring the performance of the Ribbit virtual machine in Scheme
# when bootstrapping the Scheme-to-Ribbit compiler in both
# release and debug mode.

echo "$(date)"

echo "Compiling rsc with ./rsc..."

./rsc -t scm -l max -o test-rsc.scm rsc.scm
gsc -exe test-rsc.scm
timestamp_begin="$(date +%s)"
./rsc -t scm -l max -c "./test-rsc" -o rsc-result.scm rsc.scm > test-rsc.trace.txt
timestamp_end="$(date +%s)"

echo -n "Elapsed time: "
echo -n $(($timestamp_end - $timestamp_begin))
echo ' seconds'


echo "Compiling debugging rsc with ./rsc..."

./rsc -t scm -l max -o dbg-rsc.scm rsc.scm
sed -f turn-on-debug.sed dbg-rsc.scm > dbg-rsc.scm
gsc -exe dbg-rsc.scm
timestamp_begin="$(date +%s)"
./rsc -t scm -l max -c "./dbg-rsc" -o rsc-result.scm rsc.scm > dbg-rsc.trace.txt
timestamp_end="$(date +%s)"


echo -n "Elapsed time: "
echo -n $(($timestamp_end - $timestamp_begin))
echo ' seconds'


echo "Compiling rsc with gsi rsc.scm..."

gsi rsc.scm -t scm -l max -o test-rsc.scm rsc.scm
gsc -exe test-rsc.scm
timestamp_begin="$(date +%s)"
./rsc -t scm -l max -c "./test-rsc" -o rsc-result.scm rsc.scm > test-rsc.trace.txt
timestamp_end="$(date +%s)"

echo -n "Elapsed time: "
echo -n $(($timestamp_end - $timestamp_begin))
echo ' seconds'


echo "Compiling debugging rsc with gsi rsc.scm..."

gsi rsc.scm -t scm -l max --enable-feature debug -o dbg-rsc.scm rsc.scm
gsc -exe dbg-rsc.scm
timestamp_begin="$(date +%s)"
./rsc -t scm -l max -c "./dbg-rsc" -o rsc-result.scm rsc.scm > dbg-rsc.trace.txt
timestamp_end="$(date +%s)"


echo -n "Elapsed time: "
echo -n $(($timestamp_end - $timestamp_begin))
echo ' seconds'

rm test-rsc dbg-rsc

echo 'Done'
