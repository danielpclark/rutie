echo EXPORTS >> exports.def && for /f "skip=19 tokens=4" %%A in (exports.txt) do echo %%A >> exports.def
