#!/usr/bin/env bash

set -eu

echo "type,client,tx,amount"
for i in $(seq 1 "$1"); do
	method=$(( (RANDOM % 5) + 1 ))
	client=$(( (RANDOM % 50000) + 1 ))
	tx="$((RANDOM % 100000)).$((RANDOM % 10000))"
	case "$method" in
		1)
			echo "deposit,${client},${i},${tx}"
			;;

		2)
			echo "withdrawal,${client},${i},${tx}"
			;;

		3)
			echo "dispute,${client},${i},"
			;;

		4)
			echo "resolve,${client},${i},"
			;;

		5)
			echo "chargeback,${client},${i},"
			;;
	esac
done
