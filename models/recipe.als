sig Recipe {
    , desc: String
    , title: String
    , ingredients: set Ingredient
}

sig Cat {
    , name: String
}

abstract sig Unit { }

sig Tsp extends Unit {}
sig Tbsp extends Unit {}
sig Cup extends Unit {}
sig Pint extends Unit {}
sig Quart extends Unit {}
sig Gallon extends Unit {}

abstract sig Wgt extends Unit {

}
sig MilliGram extends Wgt {}
sig Gram extends Wgt {}
sig KiloGram extends Wgt {}

abstract sig Amt {}
sig Count extends Amt {
    , value: Int
}

sig Frac extends Amt {
    , numerator: Int
    , denominator: Int
}

sig Ingredient {
    , category: Cat
    , name: String
    , unit: Unit
    , amt: Amt
}