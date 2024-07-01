// Copyright 2021 Jeremy Wall
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
/*!
Numerical measures for Recipe ingredients with math defined for them.
Each of these types implement copy and have the common math operations
defined for them.
*/

use std::{
    cmp::{Ordering, PartialEq, PartialOrd},
    convert::TryFrom,
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
    rc::Rc,
};

use num_rational::Ratio;

#[derive(Copy, Clone, Debug, PartialOrd, Eq, Ord)]
/// Volume Measurements for ingredients in a recipe.
pub enum VolumeMeasure {
    // Imperial volume measurements. US.
    /// Teaspoon measurements.
    Tsp(Quantity), // 5 ml
    /// Tablespoon measurements.
    Tbsp(Quantity), // 15 ml
    /// Cup measurements.
    Cup(Quantity), // 240 ml
    /// Pint Measurements
    Pint(Quantity), // 475 ml
    /// Quart measurements
    Qrt(Quantity), // 950 ml
    /// Gallon Measurements
    Gal(Quantity), // 3800 ml
    /// Fluid Ounces
    Floz(Quantity), // 30 ml
    // Metric volume measurements.
    /// Milliliter Measurements.
    ML(Quantity), // Base unit
    // Liter Measurements.
    Ltr(Quantity), // 1000 ml
}
use VolumeMeasure::{Cup, Floz, Gal, Ltr, Pint, Qrt, Tbsp, Tsp, ML};

// multiplier contants for various units into milliliter. Used in conversion functions.
const TSP: Quantity = Quantity::Whole(5);
const TBSP: Quantity = Quantity::Whole(15);
const FLOZ: Quantity = Quantity::Whole(30);
const CUP: Quantity = Quantity::Whole(240);
const PINT: Quantity = Quantity::Whole(480);
const QRT: Quantity = Quantity::Whole(960);
const LTR: Quantity = Quantity::Whole(1000);
const GAL: Quantity = Quantity::Whole(3840);

// multiplier constants for various units into grams
const LB: Quantity = Quantity::Frac(Ratio::new_raw(4535924, 10000));
const OZ: Quantity = Quantity::Frac(Ratio::new_raw(2834952, 100000));
const KG: Quantity = Quantity::Whole(1000);

const ONE: Quantity = Quantity::Whole(1);

impl VolumeMeasure {
    /// Get this measures `Quantity` as milliliters.
    pub fn get_ml(&self) -> Quantity {
        match self {
            ML(qty) => *qty,
            Tsp(qty) => *qty * TSP,
            Tbsp(qty) => *qty * TBSP,
            Floz(qty) => *qty * FLOZ,
            Cup(qty) => *qty * CUP,
            Pint(qty) => *qty * PINT,
            Qrt(qty) => *qty * QRT,
            Gal(qty) => *qty * GAL,
            Ltr(qty) => *qty * LTR,
        }
    }

    pub fn metric(&self) -> bool {
        match self {
            ML(_) | Ltr(_) => true,
            _ => false,
        }
    }

    pub fn plural(&self) -> bool {
        match self {
            Tsp(qty) | Tbsp(qty) | Cup(qty) | Pint(qty) | Qrt(qty) | Gal(qty) | Floz(qty)
            | ML(qty) | Ltr(qty) => qty.plural(),
        }
    }

    /// Convert into milliliters.
    pub fn into_ml(self) -> Self {
        ML(self.get_ml())
    }

    /// Convert into teaspoons.
    pub fn into_tsp(self) -> Self {
        Tsp(self.get_ml() / TSP)
    }

    /// Convert into tablespoons.
    pub fn into_tbsp(self) -> Self {
        Tbsp(self.get_ml() / TBSP)
    }

    /// Convert into fluid oz.
    pub fn into_floz(self) -> Self {
        Floz(self.get_ml() / FLOZ)
    }

    /// Convert into cups.
    pub fn into_cup(self) -> Self {
        Cup(self.get_ml() / CUP)
    }

    /// Convert into pints.
    pub fn into_pint(self) -> Self {
        Pint(self.get_ml() / PINT)
    }

    /// Convert into quarts.
    pub fn into_qrt(self) -> Self {
        Qrt(self.get_ml() / QRT)
    }

    /// Convert into gallons.
    pub fn into_gal(self) -> Self {
        Gal(self.get_ml() / GAL)
    }

    /// Convert into liters.
    pub fn into_ltr(self) -> Self {
        Ltr(self.get_ml() / LTR)
    }

    pub fn normalize(&self) -> Self {
        // We try to maintain metric vs not metric in our normalization logic.
        let metric = self.metric();
        let ml = self.get_ml();
        if (ml / GAL) >= ONE && !metric {
            return self.clone().into_gal();
        }
        if (ml / LTR) >= ONE && metric {
            return self.clone().into_ltr();
        }
        if (ml / QRT) >= ONE && !metric {
            return self.clone().into_qrt();
        }
        if (ml / PINT) >= ONE && !metric {
            return self.clone().into_pint();
        }
        if (ml / CUP) >= ONE && !metric {
            return self.clone().into_cup();
        }
        if (ml / TBSP) >= ONE && !metric {
            return self.clone().into_tbsp();
        }
        if (ml / TSP) >= ONE && !metric {
            return self.clone().into_tsp();
        }
        return if metric {
            self.clone().into_ml()
        } else {
            self.clone()
        };
    }
}

macro_rules! volume_op {
    ($trait:ident, $method:ident) => {
        impl $trait for &VolumeMeasure {
            type Output = VolumeMeasure;

            fn $method(self, lhs: Self) -> Self::Output {
                let (l, r) = (self.get_ml(), lhs.get_ml());
                let result = ML($trait::$method(l, r));
                if self.metric() {
                    result.normalize()
                } else {
                    result.into_tsp().normalize()
                }
            }
        }

        impl $trait for VolumeMeasure {
            type Output = Self;

            fn $method(self, lhs: Self) -> Self::Output {
                let (l, r) = (self.get_ml(), lhs.get_ml());
                let result = ML($trait::$method(l, r));
                if self.metric() {
                    result.normalize()
                } else {
                    result.into_tsp().normalize()
                }
            }
        }
    };
}

volume_op!(Add, add);
volume_op!(Sub, sub);

impl PartialEq for VolumeMeasure {
    fn eq(&self, lhs: &Self) -> bool {
        let rhs = self.get_ml();
        let lhs = lhs.get_ml();
        PartialEq::eq(&rhs, &lhs)
    }
}

impl Display for VolumeMeasure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tsp(qty) => write!(f, "{} tsp{}", qty, if qty.plural() { "s" } else { "" }),
            Tbsp(qty) => write!(f, "{} tbsp{}", qty, if qty.plural() { "s" } else { "" }),
            Cup(qty) => write!(f, "{} cup{}", qty, if qty.plural() { "s" } else { "" }),
            Pint(qty) => write!(f, "{} pint{}", qty, if qty.plural() { "s" } else { "" }),
            Qrt(qty) => write!(f, "{} qrt{}", qty, if qty.plural() { "s" } else { "" }),
            Gal(qty) => write!(f, "{} gal{}", qty, if qty.plural() { "s" } else { "" }),
            Floz(qty) => write!(f, "{} floz", qty),
            ML(qty) => write!(f, "{} ml", qty),
            Ltr(qty) => write!(f, "{} ltr", qty),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialOrd, Eq, Ord)]
pub enum WeightMeasure {
    Gram(Quantity),
    Kilogram(Quantity),
    Pound(Quantity),
    Oz(Quantity),
}

impl WeightMeasure {
    pub fn get_grams(&self) -> Quantity {
        match self {
            &Self::Gram(ref qty) => *qty,
            &Self::Kilogram(ref qty) => *qty * KG,
            &Self::Pound(ref qty) => *qty * LB,
            &Self::Oz(ref qty) => *qty * OZ,
        }
    }

    pub fn metric(&self) -> bool {
        match self {
            Gram(_) | Kilogram(_) => true,
            _ => false,
        }
    }

    pub fn plural(&self) -> bool {
        match self {
            &Self::Gram(qty) | &Self::Kilogram(qty) | &Self::Pound(qty) | &Self::Oz(qty) => {
                qty.plural()
            }
        }
    }

    pub fn into_gram(self) -> Self {
        Self::Gram(self.get_grams())
    }

    pub fn into_kilo(self) -> Self {
        Self::Kilogram(self.get_grams() / KG)
    }

    pub fn into_pound(self) -> Self {
        Self::Pound(self.get_grams() / LB)
    }

    pub fn into_oz(self) -> Self {
        Self::Oz(self.get_grams() / OZ)
    }

    pub fn normalize(&self) -> Self {
        let metric = self.metric();
        let grams = self.get_grams();
        if (grams / KG) >= ONE && metric {
            return self.clone().into_kilo();
        }
        if (grams / LB) >= ONE && !metric {
            return self.clone().into_pound();
        }
        if (grams / OZ) >= ONE && !metric {
            return self.clone().into_oz();
        }
        return if metric {
            self.clone().into_gram()
        } else {
            self.clone()
        };
    }
}

macro_rules! weight_op {
    ($trait:ident, $method:ident) => {
        impl $trait for &WeightMeasure {
            type Output = WeightMeasure;

            fn $method(self, lhs: Self) -> Self::Output {
                let (l, r) = (self.get_grams(), lhs.get_grams());
                let result = WeightMeasure::Gram($trait::$method(l, r));
                if self.metric() {
                    result.normalize()
                } else {
                    result.into_oz().normalize()
                }
            }
        }

        impl $trait for WeightMeasure {
            type Output = Self;

            fn $method(self, lhs: Self) -> Self::Output {
                let (l, r) = (self.get_grams(), lhs.get_grams());
                let result = WeightMeasure::Gram($trait::$method(l, r));
                if self.metric() {
                    result.normalize()
                } else {
                    result.into_oz().normalize()
                }
            }
        }
    };
}

weight_op!(Add, add);
weight_op!(Sub, sub);

impl PartialEq for WeightMeasure {
    fn eq(&self, lhs: &Self) -> bool {
        let rhs = self.get_grams();
        let lhs = lhs.get_grams();
        PartialEq::eq(&rhs, &lhs)
    }
}

impl Display for WeightMeasure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &Self::Gram(qty) => write!(f, "{} gram{}", qty, if qty.plural() { "s" } else { "" }),
            &Self::Kilogram(qty) => {
                write!(f, "{} kilogram{}", qty, if qty.plural() { "s" } else { "" })
            }
            &Self::Pound(qty) => write!(f, "{} lb{}", qty, if qty.plural() { "s" } else { "" }),
            &Self::Oz(qty) => write!(f, "{} oz", qty),
        }
    }
}

use WeightMeasure::{Gram, Kilogram, Oz, Pound};

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
/// Measurements in a Recipe with associated units for them.
pub enum Measure {
    /// Volume measurements as meter cubed base unit
    Volume(VolumeMeasure),
    /// Simple count of items
    Count(Quantity),
    Package(Rc<str>, Quantity),
    /// Weight measure as Grams base unit
    Weight(WeightMeasure),
}

use Measure::{Count, Package, Volume, Weight};

impl Measure {
    pub fn tsp(qty: Quantity) -> Self {
        Volume(Tsp(qty))
    }

    pub fn tbsp(qty: Quantity) -> Self {
        Volume(Tbsp(qty))
    }

    pub fn floz(qty: Quantity) -> Self {
        Volume(Floz(qty))
    }

    pub fn ml(qty: Quantity) -> Self {
        Volume(ML(qty))
    }

    pub fn ltr(qty: Quantity) -> Self {
        Volume(Ltr(qty))
    }

    pub fn cup(qty: Quantity) -> Self {
        Volume(Cup(qty))
    }

    pub fn qrt(qty: Quantity) -> Self {
        Volume(Qrt(qty))
    }

    pub fn pint(qty: Quantity) -> Self {
        Volume(Pint(qty))
    }

    pub fn gal(qty: Quantity) -> Self {
        Volume(Gal(qty))
    }

    pub fn count(qty: u32) -> Self {
        Count(Whole(qty))
    }

    pub fn gram(qty: Quantity) -> Self {
        Weight(Gram(qty))
    }

    pub fn kilogram(qty: Quantity) -> Self {
        Weight(Kilogram(qty))
    }

    pub fn lb(qty: Quantity) -> Self {
        // This is an approximation
        Weight(Pound(qty))
    }

    pub fn oz(qty: Quantity) -> Self {
        // This is an approximation
        Weight(Oz(qty))
    }

    pub fn pkg<S: Into<Rc<str>>>(name: S, qty: Quantity) -> Self {
        Package(name.into(), qty)
    }

    pub fn measure_type(&self) -> String {
        match self {
            Volume(_) => "Volume",
            Count(_) => "Count",
            Weight(_) => "Weight",
            Package(_, _) => "Package",
        }
        .to_owned()
    }

    pub fn plural(&self) -> bool {
        match self {
            Volume(vm) => vm.plural(),
            Count(qty) => qty.plural(),
            Weight(wm) => wm.plural(),
            Package(_, qty) => qty.plural(),
        }
    }

    pub fn normalize(&self) -> Self {
        match self {
            Volume(vm) => Volume(vm.normalize()),
            Count(qty) => Count(qty.clone()),
            Weight(wm) => Weight(wm.normalize()),
            Package(nm, qty) => Package(nm.clone(), qty.clone()),
        }
    }
}

impl Display for Measure {
    fn fmt(&self, w: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Volume(vm) => write!(w, "{}", vm),
            Count(qty) => write!(w, "{}", qty),
            Weight(wm) => write!(w, "{}", wm),
            Package(nm, qty) => write!(w, "{} {}", qty, nm),
        }
    }
}

/// Represents a Quantity for an ingredient of a recipe.
#[derive(Copy, Clone, Debug, Eq, Ord)]
pub enum Quantity {
    /// Whole or non fractional quantities of an ingredient in a recipe.
    Whole(u32),
    /// Fractional quantities of an ingredient in a recipe.
    Frac(Ratio<u32>),
}

impl Quantity {
    /// Construct a `Whole` quantity.
    pub fn whole(n: u32) -> Self {
        Whole(n)
    }

    /// Construct a Fractional quantity.
    pub fn frac(whole: u32, numer: u32, denom: u32) -> Self {
        Frac(Ratio::from_integer(whole) + Ratio::new(numer, denom))
    }

    /// For `Frac` values if the `Quantity` is a whole number normalize the `Whole(n)` type.
    /// Otherwise leave the `Quantity` untouched.
    pub fn normalize(self) -> Self {
        if let Frac(rat) = self {
            if rat.is_integer() {
                Whole(*rat.numer())
            } else {
                Frac(rat)
            }
        } else {
            self
        }
    }

    /// Extract out the whole and the fractional parts of a `Quantity`.
    pub fn extract_parts(self) -> (u32, Ratio<u32>) {
        match self {
            Whole(v) => (v, Ratio::new(0, 1)),
            Frac(v) => (v.to_integer(), v.fract()),
        }
    }

    /// Approximate a quantity as a float. This will lose precision in the case
    /// of fractional quantities.
    pub fn approx_f32(self) -> f32 {
        match self {
            Whole(v) => v as f32,
            Frac(v) => (*v.numer() / *v.denom()) as f32,
        }
    }

    pub fn plural(&self) -> bool {
        match self {
            Whole(v) => *v > 1,
            Frac(r) => *r > Ratio::new(1, 1),
        }
    }
}
use Quantity::{Frac, Whole};

pub struct ConversionError {
    pub err_message: String,
}

impl From<Ratio<u32>> for Quantity {
    fn from(r: Ratio<u32>) -> Self {
        Quantity::Frac(r).normalize()
    }
}

impl From<u32> for Quantity {
    fn from(u: u32) -> Self {
        Quantity::Whole(u)
    }
}

impl TryFrom<f32> for Quantity {
    type Error = ConversionError;

    fn try_from(f: f32) -> std::result::Result<Self, Self::Error> {
        Ratio::approximate_float(f)
            .map(|rat: Ratio<i32>| Frac(Ratio::new(*rat.numer() as u32, *rat.denom() as u32)))
            .ok_or_else(|| ConversionError {
                err_message: format!("Cannot Convert {} into a Rational", f),
            })
    }
}

macro_rules! quantity_op {
    ($trait:ident, $method:ident) => {
        impl $trait for &Quantity {
            type Output = Quantity;

            fn $method(self, lhs: Self) -> Self::Output {
                match (self, lhs) {
                    (Whole(rhs), Whole(lhs)) => Frac($trait::$method(
                        Ratio::from_integer(*rhs),
                        Ratio::from_integer(*lhs),
                    )),
                    (Frac(rhs), Frac(lhs)) => Frac($trait::$method(rhs, lhs)),
                    (Whole(rhs), Frac(lhs)) => {
                        Frac($trait::$method(Ratio::from_integer(*rhs), lhs))
                    }
                    (Frac(rhs), Whole(lhs)) => {
                        Frac($trait::$method(rhs, Ratio::from_integer(*lhs)))
                    }
                }
            }
        }

        impl $trait for Quantity {
            type Output = Self;

            fn $method(self, lhs: Self) -> Self::Output {
                match (self, lhs) {
                    (Whole(rhs), Whole(lhs)) => Frac($trait::$method(
                        Ratio::from_integer(rhs),
                        Ratio::from_integer(lhs),
                    )),
                    (Frac(rhs), Frac(lhs)) => Frac($trait::$method(rhs, lhs)),
                    (Whole(rhs), Frac(lhs)) => Frac($trait::$method(Ratio::from_integer(rhs), lhs)),
                    (Frac(rhs), Whole(lhs)) => Frac($trait::$method(rhs, Ratio::from_integer(lhs))),
                }
            }
        }
    };
}

quantity_op!(Add, add);
quantity_op!(Sub, sub);
quantity_op!(Mul, mul);
quantity_op!(Div, div);

impl PartialOrd for Quantity {
    fn partial_cmp(&self, lhs: &Self) -> Option<Ordering> {
        match (self, lhs) {
            (Whole(rhs), Whole(lhs)) => PartialOrd::partial_cmp(rhs, lhs),
            (Frac(rhs), Frac(lhs)) => PartialOrd::partial_cmp(rhs, lhs),
            (Whole(rhs), Frac(lhs)) => PartialOrd::partial_cmp(&Ratio::from_integer(*rhs), lhs),
            (Frac(rhs), Whole(lhs)) => PartialOrd::partial_cmp(rhs, &Ratio::from_integer(*lhs)),
        }
    }
}

impl PartialEq for Quantity {
    fn eq(&self, lhs: &Self) -> bool {
        match (self, lhs) {
            (Whole(rhs), Whole(lhs)) => PartialEq::eq(rhs, lhs),
            (Frac(rhs), Frac(lhs)) => PartialEq::eq(rhs, lhs),
            (Whole(rhs), Frac(lhs)) => PartialEq::eq(&Ratio::from_integer(*rhs), lhs),
            (Frac(rhs), Whole(lhs)) => PartialEq::eq(rhs, &Ratio::from_integer(*lhs)),
        }
    }
}

impl Display for Quantity {
    fn fmt(&self, w: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.normalize() {
            Whole(v) => write!(w, "{}", v),
            Frac(_) => {
                let (whole, frac) = self.extract_parts();
                if whole == 0 {
                    write!(w, "{}/{}", frac.numer(), frac.denom())
                } else {
                    write!(w, "{} {}/{}", whole, frac.numer(), frac.denom())
                }
            }
        }
    }
}
