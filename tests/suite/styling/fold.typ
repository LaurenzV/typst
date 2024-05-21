--- fold-vec-order-text-features ---
// Test fold order of vectors.
#set text(features: (liga: 1))
#set text(features: (liga: 0))
fi

--- fold-vec-order-text-decos ---
#underline(stroke: aqua + 4pt)[
  #underline[Hello]
]

--- fold-vec-order-meta ---
#let c = counter("mycounter")
#c.update(1)
#locate(loc => [
  #c.update(2)
  #c.at(loc) \
  Second: #locate(loc => c.at(loc))
])
