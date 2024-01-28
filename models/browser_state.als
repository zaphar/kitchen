sig Id {}
sig Text {}

sig Recipe {
 	, id: one Id
   , text: one Text
} 

fact {
   no r1, r2: Recipe | (r1.id = r2.id) and (r1.text != r2.text)
   no r1, r2: Recipe | (r1 != r2) and (r1.id = r2.id)
}

sig Ingredient {}
sig Modifier {}
sig Amt {}

sig ModifiedInventory {
    , ingredient: one Ingredient
    , modifier: lone Modifier
    , amt: one Amt
}

fact {
   no mi1, mi2: ModifiedInventory | mi1 != mi2 && (mi1.ingredient = mi2.ingredient) and (mi1.modifier = mi2.modifier)
}

sig DeletedInventory {
    , ingredient: one Ingredient
    , modifier: lone Modifier
}

fact {
   no mi1, mi2: DeletedInventory | mi1 != mi2 && (mi1.ingredient = mi2.ingredient) and (mi1.modifier = mi2.modifier)
}

sig ExtraItems {
    , ingredient: one Ingredient
    , amt: one Amt
}

sig State {
	, recipes: some Recipe
    , modified: set ModifiedInventory
    , deleted: set DeletedInventory
    , extras: set ExtraItems
} {
	no rs: Recipe | rs not in recipes
}

run { } for 3 but exactly 2 State, 2 Modifier, exactly 3 ModifiedInventory, exactly 9 Ingredient
