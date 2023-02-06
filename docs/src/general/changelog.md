---
description: |
  Learn what has changed in the latest Typst releases and move your documents
  forward.
---

# Changelog
## February 2, 2023
- Merged text and math symbols, renamed a few symbols
  (including `infty` to `infinity` with the alias `oo`)
- Fixed missing italic mappings
- Math italics correction is now applied properly
- Parentheses now scale in `[$zeta(x/2)$]`
- Fixed placement of large root index
- Fixed spacing in `[$abs(-x)$]`
- Fixed inconsistency between text and identifiers in math
- Accents are now ignored when positioning superscripts
- Fixed vertical alignment in matrices
- Fixed `text` set rule in `raw` show rule
- Heading and list markers now parse consistently
- Allow arbitrary math directly in content

## January 30, 2023
[Go to the announcement blog post.](https://typst.app/blog/2023/january-update)
- New expression syntax in markup/math
  - Blocks cannot be directly embedded in markup anymore
  - Like other expressions, they now require a leading hashtag
  - More expressions available with hashtag, including literals
    (`[#"string"]`) as well as field access and method call
    without space: `[#emoji.face]`
- New import syntax
  - `[#import "module.typ"]` creates binding named `module`
  - `[#import "module.typ": a, b]` or `[#import "module.typ": *]` to import items
  - `[#import emoji: face, turtle]` to import from already bound module
- New symbol handling
  - Removed symbol notation
  - Symbols are now in modules: `{sym}`, `{emoji}`, and `{math}`
  - Math module also reexports all of `{sym}`
  - Modified through field access, still order-independent
  - Unknown modifiers are not allowed anymore
  - Support for custom symbol definitions with `symbol` function
  - Symbols now listed in documentation
- New `{math}` module
  - Contains all math-related functions
  - Variables and function calls directly in math (without hashtag) access this
    module instead of the global scope, but can also access local variables
  - Can be explicitly used in code, e.g. `[#set math.vec(delim: "[")]`
- Delimiter matching in math
   - Any opening delimiters matches any closing one
   - When matched, they automatically scale
   - To prevent scaling, escape them
   - To forcibly match two delimiters, use `lr` function
   - Line breaks may occur between matched delimiters
   - Delimiters may also be unbalanced
   - You can also use the `lr` function to scale the brackets
     (or just one bracket) to a specific size manually
- Multi-line math with alignment
  - The `\` character inserts a line break
  - The `&` character defines an alignment point
  - Alignment points also work for underbraces, vectors, cases, and matrices
  - Multiple alignment points are supported
- More capable math function calls
  - Function calls directly in math can now take code expressions with hashtag
  - They can now also take named arguments
  - Within math function calls, semicolons turn preceding arguments to arrays to
    support matrices: `[$mat(1, 2; 3, 4)$]`
- Arbitrary content in math
  - Text, images, and other arbitrary content can now be embedded in math
  - Math now also supports font fallback to support e.g. CJK and emoji
- More math features
  - New text operators: `op` function, `lim`, `max`, etc.
  - New matrix function: `mat`
  - New n-ary roots with `root` function: `[$root(3, x)$]`
  - New under- and overbraces, -brackets, and -lines
  - New `abs` and `norm` functions
  - New shorthands: `[|`, `|]`, and `||`
  - New `attach` function, overridable attachments with `script` and `limit`
  - Manual spacing in math, with `h`, `thin`, `med`, `thick` and `quad`
  - Symbols and other content may now be used like a function, e.g. `[$zeta(x)$]`
  - Added Fira Math font, removed Noto Sans Math font
  - Support for alternative math fonts through
    `[#show math.formula: set text("Fira Math")]`
- More library improvements
  - New `calc` module, `abs`, `min`, `max`, `even`, `odd` and `mod` moved there
  - New `message` argument on `{assert}` function
  - The `pairs` method on dictionaries now returns an array of length-2 arrays
    instead of taking a closure
  - The method call `{dict.at("key")}` now always fails if `"key"` doesn't exist
    Previously, it was allowed in assignments. Alternatives are `{dict.key = x}`
    and `{dict.insert("key", x)}`.
- Smarter editor functionality
  - Autocompletion for local variables
  - Autocompletion for methods available on a value
  - Autocompletion for symbols and modules
  - Autocompletion for imports
  - Hover over an identifier to see its value(s)
- Further editor improvements
  - New Font menu with previews
  - Single projects may now be shared with share links
  - New dashboard experience if projects are shared with you
  - Keyboard Shortcuts are now listed in the menus and there are more of them
  - New Offline indicator
  - Tooltips for all buttons
  - Improved account protection
  - Moved Status indicator into the error list button
- Fixes
  - Multiple bug fixes for incremental parser
  - Fixed closure parameter capturing
  - Fixed tons of math bugs
  - Bugfixes for performance, file management, editing reliability
  - Added redirection to the page originally navigated to after signin