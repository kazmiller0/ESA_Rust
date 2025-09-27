//! Implements a dynamic cryptographic accumulator that supports additions and deletions.

use super::{
    utils::{digest_to_prime_field, xgcd},
    Curve, Fr, G1Affine, G1Projective, G2Affine, G2Projective,
};
use crate::digest::Digestible;
use anyhow::{anyhow, Result};
use ark_ec::{AffineCurve, PairingEngine, ProjectiveCurve};
use ark_ff::{Field, One, PrimeField, Zero};
use ark_poly::{univariate::DensePolynomial, Polynomial, UVPolynomial};
use std::collections::HashSet;
use std::ops::Neg;

/// A proof that an 'add' operation was performed correctly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddProof {
    pub old_acc_value: G1Affine,
    pub new_acc_value: G1Affine,
    pub element: Fr,
}

impl AddProof {
    /// Verifies that the new accumulator is the result of adding the element to the old one.
    /// It checks if e(new_acc, g2) == e(old_acc, g2^(s-element)).
    pub fn verify(&self) -> bool {
        // Calculate g2^(s-element)
        let s_minus_elem: Fr = *super::PRI_S - self.element;
        let g2_s_minus_elem = super::G2_POWER.apply(&s_minus_elem);

        let lhs = Curve::pairing(self.new_acc_value, G2Affine::prime_subgroup_generator());
        let rhs = Curve::pairing(self.old_acc_value, g2_s_minus_elem);

        lhs == rhs
    }
}

/// A proof that a 'delete' operation was performed correctly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteProof {
    pub old_acc_value: G1Affine,
    pub new_acc_value: G1Affine,
    pub element: Fr,
}

impl DeleteProof {
    /// Verifies that the new accumulator is the result of deleting the element from the old one.
    /// It checks if e(new_acc, g2^(s-element)) == e(old_acc, g2).
    pub fn verify(&self) -> bool {
        // Calculate g2^(s-element)
        let s_minus_elem: Fr = *super::PRI_S - self.element;
        let g2_s_minus_elem = super::G2_POWER.apply(&s_minus_elem);

        let lhs = Curve::pairing(self.new_acc_value, g2_s_minus_elem);
        let rhs = Curve::pairing(self.old_acc_value, G2Affine::prime_subgroup_generator());

        lhs == rhs
    }
}

/// A proof of membership for an element in the accumulator.
/// The witness is an accumulator of the set without the element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipProof {
    pub witness: G1Affine,
    pub element: Fr,
}

impl MembershipProof {
    /// Verifies that this proof is valid for the given accumulator value.
    /// It checks if e(witness, g2^(s-element)) == e(accumulator, g2).
    pub fn verify(&self, accumulator: G1Affine) -> bool {
        // Calculate g2^(s-element)
        let s_minus_elem: Fr = *super::PRI_S - self.element;
        let g2_s_minus_elem = super::G2_POWER.apply(&s_minus_elem);

        let lhs = Curve::pairing(self.witness, g2_s_minus_elem);
        let rhs = Curve::pairing(accumulator, G2Affine::prime_subgroup_generator());

        lhs == rhs
    }
}

/// A proof of non-membership for an element in the accumulator.
/// This proof shows that the element is not in the set represented by the accumulator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonMembershipProof {
    pub element: Fr,
    /// Witness for non-membership, g2^B(s)
    pub witness: G2Affine,
    /// g1^A(s), the other part of the proof
    pub g1_a: G1Affine,
}

/// Represents the result of a query against the accumulator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryResult {
    /// The element is in the set, and here is the proof.
    Membership(MembershipProof),
    /// The element is not in the set, and here is the proof.
    NonMembership(NonMembershipProof),
}

/// A dynamic cryptographic accumulator based on the Acc1 scheme.
/// It maintains the accumulator value and the set of elements internally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicAccumulator {
    /// The current accumulator value, g1^P(s).
    pub acc_value: G1Affine,
    /// The set of elements (as field elements).
    elements: HashSet<Fr>,
}

impl DynamicAccumulator {
    /// Creates a new, empty dynamic accumulator.
    /// The initial value is g1^1, representing an empty set.
    pub fn new() -> Self {
        Self {
            acc_value: G1Projective::from(G1Affine::prime_subgroup_generator())
                .mul(Fr::one().into_repr())
                .into_affine(),
            elements: HashSet::new(),
        }
    }

    /// Adds a new element to the accumulator and returns a proof of the operation.
    /// If the element already exists, it returns an error.
    /// The accumulator value is updated by scalar multiplying it with (s-element).
    pub fn add(&mut self, element: &i64) -> Result<AddProof> {
        let fr_element = digest_to_prime_field(&element.to_digest());
        if self.elements.contains(&fr_element) {
            return Err(anyhow!("Element already in accumulator"));
        }
        let old_acc = self.acc_value;

        // Update accumulator value: acc' = acc^(s-element)
        let s_minus_elem: Fr = *super::PRI_S - fr_element;
        self.acc_value = self
            .acc_value
            .into_projective()
            .mul(s_minus_elem.into_repr())
            .into_affine();

        // Update the element set
        self.elements.insert(fr_element);

        Ok(AddProof {
            old_acc_value: old_acc,
            new_acc_value: self.acc_value,
            element: fr_element,
        })
    }

    /// Updates an element in the accumulator from an old value to a new one.
    /// This is implemented as a delete operation followed by an add operation.
    /// Returns proofs for both operations.
    /// Returns an error if the old element is not in the accumulator.
    pub fn update(&mut self, old_element: &i64, new_element: &i64) -> Result<(DeleteProof, AddProof)> {
        let delete_proof = self.delete(old_element)?;
        let add_proof = self.add(new_element)?;
        Ok((delete_proof, add_proof))
    }

    /// Deletes an element from the accumulator and returns a proof of the operation.
    /// If the element exists, its count is decremented. If the count reaches zero, it's removed.
    /// The accumulator value is updated by scalar multiplying it with the inverse of (s-element).
    /// Returns an error if the element is not in the accumulator.
    pub fn delete(&mut self, element: &i64) -> Result<DeleteProof> {
        let fr_element = digest_to_prime_field(&element.to_digest());
        let old_acc = self.acc_value;

        if !self.elements.contains(&fr_element) {
            return Err(anyhow!("Element not in accumulator"));
        }

        // Update accumulator value: acc' = acc^((s-element)^-1)
        let s_minus_elem: Fr = *super::PRI_S - fr_element;
        let s_minus_elem_inv = s_minus_elem
            .inverse()
            .ok_or_else(|| anyhow!("Failed to compute inverse"))?;
        self.acc_value = self
            .acc_value
            .into_projective()
            .mul(s_minus_elem_inv.into_repr())
            .into_affine();

        // Update the element set
        self.elements.remove(&fr_element);

        Ok(DeleteProof {
            old_acc_value: old_acc,
            new_acc_value: self.acc_value,
            element: fr_element,
        })
    }

    /// Generates a membership proof for a given element.
    /// The proof's witness is an accumulator for the set of all other elements.
    /// Returns an error if the element is not in the accumulator.
    pub fn prove_membership(&self, element: &i64) -> Result<MembershipProof> {
        let fr_element = digest_to_prime_field(&element.to_digest());

        if !self.elements.contains(&fr_element) {
            return Err(anyhow!(
                "Cannot prove membership for an element not in the set"
            ));
        }

        // Calculate witness: acc^((s-element)^-1)
        let s_minus_elem: Fr = *super::PRI_S - fr_element;
        let s_minus_elem_inv = s_minus_elem
            .inverse()
            .ok_or_else(|| anyhow!("Failed to compute inverse"))?;
        let witness = self
            .acc_value
            .into_projective()
            .mul(s_minus_elem_inv.into_repr())
            .into_affine();

        Ok(MembershipProof {
            witness,
            element: fr_element,
        })
    }

    /// Verifies a membership proof against the current accumulator value.
    pub fn verify_membership(&self, proof: &MembershipProof) -> bool {
        proof.verify(self.acc_value)
    }

    /// Generates a non-membership proof for a given element.
    /// Returns an error if the element IS in the accumulator.
    pub fn prove_non_membership(&self, element: &i64) -> Result<NonMembershipProof> {
        let fr_element = digest_to_prime_field(&element.to_digest());

        if self.elements.contains(&fr_element) {
            return Err(anyhow!(
                "Cannot prove non-membership for an element in the set"
            ));
        }

        // To prove x is not in E, we show that gcd(P(X), X-x) = 1, where P(X) = product(X-e_i).
        // Using XGCD, we find polynomials A(X), B(X) such that A(X)*(X-x) + B(X)*P(X) = 1.
        // The proof is (g1^A(s), g2^B(s)).
        // Verification checks e(Acc, g2^B(s)) * e(g1^A(s), g2^(s-x)) == e(g1, g2)
        // This corresponds to e(g1^P(s), g2^B(s)) * e(g1^A(s), g2^(s-x)) == e(g1, g2)
        // which is B(s)*P(s) + A(s)*(s-x) = 1.

        // 1. Construct the accumulator polynomial P(X) = product(X-e_i).
        let mut p_poly = DensePolynomial::from_coefficients_vec(vec![Fr::one()]);
        for elem in &self.elements {
            // X - e
            let e_poly = DensePolynomial::from_coefficients_vec(vec![elem.neg(), Fr::one()]);
            p_poly = &p_poly * &e_poly;
        }

        // 2. Construct the polynomial for the non-member, Q(X) = X-x.
        let q_poly = DensePolynomial::from_coefficients_vec(vec![fr_element.neg(), Fr::one()]); // X-x

        // 3. Run XGCD on Q(X) and P(X) to find A(X) and B(X).
        // We want A(X)*Q(X) + B(X)*P(X) = 1
        if let Some((gcd, a_poly, b_poly)) = xgcd(q_poly, p_poly.clone()) {
            // GCD must be a non-zero constant for the proof to be valid.
            if !gcd.is_zero() && gcd.degree() == 0 {
                // The equation is a_poly*Q(X) + b_poly*P(X) = gcd.
                // We need it to be 1, so we must divide by the constant value of gcd.
                let gcd_val = gcd.coeffs.get(0).cloned().unwrap_or_else(Fr::one);
                let gcd_inv = gcd_val
                    .inverse()
                    .ok_or_else(|| anyhow!("Failed to compute gcd inverse"))?;

                let a_poly_norm = DensePolynomial::from_coefficients_vec(
                    a_poly.coeffs.iter().map(|c| *c * gcd_inv).collect(),
                );
                let b_poly_norm = DensePolynomial::from_coefficients_vec(
                    b_poly.coeffs.iter().map(|c| *c * gcd_inv).collect(),
                );

                // 4. Evaluate the normalized polynomials at the secret `s`.
                let a_s = a_poly_norm.evaluate(&*super::PRI_S);
                let b_s = b_poly_norm.evaluate(&*super::PRI_S);

                // 5. Compute the witness parts: g1^A(s) and g2^B(s)
                let g1_a = G1Projective::prime_subgroup_generator()
                    .mul(a_s.into_repr())
                    .into_affine();
                let witness_b = G2Projective::prime_subgroup_generator()
                    .mul(b_s.into_repr())
                    .into_affine();

                return Ok(NonMembershipProof {
                    element: fr_element,
                    witness: witness_b, // This is g2^B(s)
                    g1_a,               // This is g1^A(s)
                });
            }
        }

        Err(anyhow!("Failed to create non-membership proof"))
    }

    /// Verifies a non-membership proof against the current accumulator value.
    pub fn verify_non_membership(&self, proof: &NonMembershipProof) -> bool {
        // Verification equation: e(Acc, witness) * e(g1_a, g2^(s-x)) == e(g1, g2)
        // Here, witness = g2^B(s) and g1_a = g1^A(s).
        // So, e(g1^P(s), g2^B(s)) * e(g1^A(s), g2^(s-x)) == e(g1, g2)
        // which simplifies to e(g1,g2)^(B(s)*P(s) + A(s)*(s-x)) == e(g1,g2)^1
        // This holds if B(s)*P(s) + A(s)*(s-x) = 1.

        // 1. Calculate g2^(s-x)
        let s_minus_x = *super::PRI_S - proof.element;
        let g2_s_minus_x = super::G2_POWER.apply(&s_minus_x);

        // 2. Calculate the pairings
        let lhs1 = Curve::pairing(self.acc_value, proof.witness);
        let lhs2 = Curve::pairing(proof.g1_a, g2_s_minus_x);
        let rhs = Curve::pairing(
            G1Affine::prime_subgroup_generator(),
            G2Affine::prime_subgroup_generator(),
        );

        lhs1 * lhs2 == rhs
    }

    /// Queries the accumulator for a given element and returns a cryptographic proof
    /// of either membership or non-membership.
    pub fn query(&self, element: &i64) -> QueryResult {
        let fr_element = digest_to_prime_field(&element.to_digest());
        if self.elements.contains(&fr_element) {
            // This unwrap is safe because we've just checked for the element's existence.
            let proof = self.prove_membership(element).unwrap();
            QueryResult::Membership(proof)
        } else {
            // This unwrap is safe because we've just checked for the element's non-existence.
            let proof = self.prove_non_membership(element).unwrap();
            QueryResult::NonMembership(proof)
        }
    }
}

impl Default for DynamicAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acc::Accumulator;
    use crate::digest::Digestible;
    use crate::{Acc1, MultiSet};

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_dynamic_accumulator_add() {
        init_logger();
        let mut dyn_acc = DynamicAccumulator::new();
        let add_proof1 = dyn_acc.add(&1i64).unwrap();
        let add_proof2 = dyn_acc.add(&2i64).unwrap();
        let add_proof3_res = dyn_acc.add(&1i64); // Add 1 again

        // Verify proofs
        assert!(add_proof1.verify());
        assert!(add_proof2.verify());
        assert!(add_proof3_res.is_err()); // Should fail to add duplicate

        let set = MultiSet::from_vec(vec![1i64, 2]);
        let static_acc = Acc1::cal_acc_g1_sk(&set);

        assert_eq!(dyn_acc.acc_value, static_acc);
        assert_eq!(dyn_acc.elements.len(), 2);
        assert!(dyn_acc
            .elements
            .contains(&digest_to_prime_field(&1i64.to_digest())));
        assert!(dyn_acc
            .elements
            .contains(&digest_to_prime_field(&2i64.to_digest())));
    }

    #[test]
    fn test_dynamic_accumulator_delete() {
        init_logger();
        let mut dyn_acc = DynamicAccumulator::new();
        dyn_acc.add(&1i64).unwrap();
        dyn_acc.add(&2i64).unwrap();

        // Delete 1
        let delete_proof1 = dyn_acc.delete(&1i64).unwrap();
        assert!(delete_proof1.verify());

        let set1 = MultiSet::from_vec(vec![2i64]);
        let static_acc1 = Acc1::cal_acc_g1_sk(&set1);
        assert_eq!(dyn_acc.acc_value, static_acc1);
        assert!(!dyn_acc
            .elements
            .contains(&digest_to_prime_field(&1i64.to_digest())));

        // Try to delete 1 again (should fail)
        assert!(dyn_acc.delete(&1i64).is_err());

        // Delete 2
        let delete_proof2 = dyn_acc.delete(&2i64).unwrap();
        assert!(delete_proof2.verify());

        let set2: MultiSet<i64> = MultiSet::from_vec(vec![]);
        let static_acc2 = Acc1::cal_acc_g1_sk(&set2);
        assert_eq!(dyn_acc.acc_value, static_acc2);
        assert!(dyn_acc.elements.is_empty());

        // Try to delete an element that was never there
        assert!(dyn_acc.delete(&3i64).is_err());
    }

    #[test]
    fn test_membership_proof() {
        init_logger();
        let mut dyn_acc = DynamicAccumulator::new();
        dyn_acc.add(&100).unwrap();
        dyn_acc.add(&200).unwrap();
        dyn_acc.add(&300).unwrap();

        // 1. Prove and verify 200
        let proof = dyn_acc.prove_membership(&200).unwrap();
        assert!(dyn_acc.verify_membership(&proof));
        assert!(proof.verify(dyn_acc.acc_value));

        // 2. Check that the witness is correct
        let set_without_200 = MultiSet::from_vec(vec![100i64, 300]);
        let witness_static = Acc1::cal_acc_g1_sk(&set_without_200);
        assert_eq!(proof.witness, witness_static);

        // 3. A proof for a different element should fail
        let mut wrong_proof = proof.clone();
        wrong_proof.element = digest_to_prime_field(&999i64.to_digest());
        assert!(!dyn_acc.verify_membership(&wrong_proof));

        // 4. Cannot prove membership for an element not in the set
        assert!(dyn_acc.prove_membership(&999i64).is_err());
    }

    #[test]
    fn test_non_membership_proof() {
        init_logger();
        let mut dyn_acc = DynamicAccumulator::new();
        dyn_acc.add(&100).unwrap();
        dyn_acc.add(&200).unwrap();

        // 1. Prove and verify non-membership for 300
        let proof = dyn_acc.prove_non_membership(&300).unwrap();
        assert!(dyn_acc.verify_non_membership(&proof));

        // 2. A non-membership proof for an element that IS in the set should fail
        assert!(dyn_acc.prove_non_membership(&100).is_err());

        // 3. A tampered proof should fail verification
        let mut tampered_proof = proof.clone();
        tampered_proof.element = digest_to_prime_field(&400i64.to_digest());
        assert!(!dyn_acc.verify_non_membership(&tampered_proof));

        // 4. An empty accumulator should be able to prove non-membership
        let empty_acc = DynamicAccumulator::new();
        let proof_for_empty = empty_acc.prove_non_membership(&100).unwrap();
        assert!(empty_acc.verify_non_membership(&proof_for_empty));
    }

    #[test]
    fn test_update_and_query() {
        init_logger();
        let mut dyn_acc = DynamicAccumulator::new();
        dyn_acc.add(&100).unwrap();
        dyn_acc.add(&200).unwrap();

        // 1. Test successful update
        let (delete_proof, add_proof) = dyn_acc.update(&100, &150).unwrap();
        assert!(delete_proof.verify());
        assert!(add_proof.verify());

        // Verify 100 is gone
        match dyn_acc.query(&100) {
            QueryResult::NonMembership(proof) => {
                assert!(dyn_acc.verify_non_membership(&proof));
            }
            _ => panic!("Should have been a non-membership proof for 100"),
        }

        // Verify 150 is present
        match dyn_acc.query(&150) {
            QueryResult::Membership(proof) => {
                assert!(dyn_acc.verify_membership(&proof));
            }
            _ => panic!("Should have been a membership proof for 150"),
        }

        // Verify 200 is still present
        match dyn_acc.query(&200) {
            QueryResult::Membership(proof) => {
                assert!(dyn_acc.verify_membership(&proof));
            }
            _ => panic!("Should have been a membership proof for 200"),
        }

        // 2. Test update on a non-existent element (should fail)
        assert!(dyn_acc.update(&999, &1000).is_err());
    }
}
