#!/bin/bash
echo "EXPORTS" > exports.def
echo "rb_cObject" >> exports.def
grep -hoRP '(?<=    pub fn )rb_[_a-z0-9]*' src/rubysys | sort - | uniq - >> exports.def
grep -hoRP '(?<=    pub fn )ruby_[_a-z0-9]*' src/rubysys | sort - | uniq - >> exports.def
