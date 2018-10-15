#!/bin/bash
echo "EXPORTS" > exports.def
grep -hoRP '(?<=    pub fn )rb_[_a-z0-9]*' src/rubysys | sort - | uniq - >> exports.def
