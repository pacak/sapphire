window.SIDEBAR_ITEMS = {"struct":[["ArenaMap","This is meant to act as a primary mapping of `K -> V`, where `K` is some key type and `V` is the value being stored. Other mappings that use the same key as an existing [`ArenaMap`] should use `SecondaryMap` instead."],["SecondaryMap","Intended to be a dense secondary mapping `K -> V` for keys from a primary [`ArenaMap`]. This is to associate extra data with most (but ideally all) keys from a given `PrimaryMap`."],["UniqueArenaMap","Contains a table of immutable, unique elements. All elements are only stored once, no duplicates are stored inside of the arena itself. Lookups by key are almost as efficient as they are in a plain [`ArenaMap`], although an extra pointer dereference must be done."]],"trait":[["ArenaKey","Models a type that can act as a key for the arena map types."],["PackableKey","Models a key type that can be used in packed data structures that require a “null” value (such as `PackedOption<T>`)."]]};