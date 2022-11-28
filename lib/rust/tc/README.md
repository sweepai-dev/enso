not my most efficient rust code ever.

Switched to Box<> internally so I could use box patterns and box expressions.

This should switch to Arc<> or a hashconsing type later, but doing so seems to make the unifier into a complete mess of spaghetti code, so it behooves us to get it right, and get it tested before making it fast.

Reverted to the simpler base hindley milner gundry style checker for now to more quickly iterate on style. Adding in the extra pieces of the unifier is easy enough now that we have the pattern.