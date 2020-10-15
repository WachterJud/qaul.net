//! Per-user contact book management
//!
//! When querying user information from libqaul, a profile contains a
//! lot of publicly available information, which is (or should be) the
//! same for all users on the network.  To allow applications to tag
//! users with additional information, the contact API provides an
//! overlay type that achieves this.

use serde::{Deserialize, Serialize};

/// Per-user local contact access on the network
///
/// All fields in this structure are entirely optional and can not be
/// relied on. They are additional points of data, that a user can
/// specify about another user, that are not available or shared with
/// the network. This is meant to allow users to curate a list of
/// trusted contacts, or build friend circles.
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ContactEntry {
    /// The name by which the associated contact is known by the owning user.
    pub nick: Option<String>,
    /// Set a user trust level
    pub trust: i8,
    /// The user has met this person
    pub met: bool,
    /// A free text location
    pub location: Option<String>,
    /// A general plain text notes section
    pub notes: Option<String>,
}

/// Query structure to find contacts by
///
/// A query is always applied to a field that is present in
/// `ContactEntry`, and will filter contacts by what set of
/// prerequisites they fulfill.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ContactQuery {
    /// A fuzzy nickname search
    Nick(String),
    /// A fuzzy trust level search
    Trust { val: i8, fuz: i8 },
    /// Filter by physical meeting
    Met(bool),
    /// A fuzzy location string search
    Location(String),
    /// A fuzzy notes string search
    Notes(String),
}
