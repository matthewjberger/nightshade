Sanity-check recent code by reasoning through it. Do not run the binary, do not run tests, do not write a small test program. Just read and reason.

1. Identify the area in question (from arguments or the most recent changes)
2. Read every file involved end-to-end, not just the touched lines
3. Trace the actual control flow and data flow:
   - For each input, what value does it hold at this point?
   - For each branch/match arm, when is it taken and is anything missed?
   - For each loop, what's the invariant and what does it produce?
   - For layout/rendering: which coordinate system, which units, which order do systems run in?
4. Look for hidden conflicts:
   - Duplicate scaling, double-counted offsets, mismatched coordinate systems
   - Two systems writing the same field
   - Lifetimes/visibility/effective-state where ancestors matter
   - Off-by-one indices, separator skips, padding accounting
   - Race between input frame and layout/render frame
5. Compare what the code does to what the user actually sees in the most recent screenshot or report
6. Report:
   - What's correct and why (concretely)
   - What's broken and why (concretely, with the exact mechanism)
   - The minimal fix, not a refactor

Do not rubber-stamp. If something looks fine but you cannot articulate why, it is not fine yet, so keep reading.

If arguments are provided, focus the sanity check on that specific area.
