//! Model-aware inspection of ChemApp excess Gibbs and magnetic interactions.
//!
//! The interaction path is deliberately additive: [`InteractionRaw`] retains
//! the exact TQLPAR text and authoritative live TQGPAR coefficients,
//! [`Interaction::native_parsed`] records how that native text parsed, and an
//! optional ASCII-DAT cross-check supplies independent source-side evidence.
//! DAT structure becomes effective only for a typed, validated native defect;
//! ordinary disagreements and provider failures leave healthy native data in
//! use. Unknown syntax is retained instead of being discarded.

use std::fmt::{self, Display, Write};

use crate::error::ChemAppError;
use crate::Engine;

/// A TQLPAR/TQGPAR interaction channel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InteractionChannel {
    /// Excess Gibbs-energy interactions (`OPTION = "G"`).
    GibbsExcess,
    /// Excess magnetic interactions (`OPTION = "M"`).
    Magnetic,
}

impl InteractionChannel {
    pub(crate) fn option(self) -> &'static str {
        match self {
            Self::GibbsExcess => "G",
            Self::Magnetic => "M",
        }
    }
}

impl Display for InteractionChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::GibbsExcess => "Gibbs excess",
            Self::Magnetic => "Magnetic",
        })
    }
}

/// Semantic role of one magnetic TQGPAR column.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MagneticInteractionRole {
    /// Column 1: excess Curie/Neel temperature.
    CurieNeelTemperature,
    /// Column 2: excess magnetic moment.
    MagneticMoment,
}

impl MagneticInteractionRole {
    fn tqcdat_selector(self) -> usize {
        match self {
            Self::CurieNeelTemperature => 1,
            Self::MagneticMoment => 2,
        }
    }
}

impl Display for MagneticInteractionRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::CurieNeelTemperature => "Curie/Neel temperature",
            Self::MagneticMoment => "Magnetic moment",
        })
    }
}

/// A one-based, channel-safe address of one live ChemApp interaction value.
///
/// Gibbs parameters lower to `TQCDAT(13, interaction, expression, term,
/// phase, value)`. Magnetic parameters lower to `TQCDAT(10, interaction,
/// expression, role, phase, value)`. The variants make it impossible to put a
/// magnetic role on a Gibbs interaction accidentally.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InteractionParameterAddress {
    /// One ordinary excess Gibbs coefficient.
    Gibbs {
        /// One-based phase index.
        phase_index: usize,
        /// One-based interaction index in TQLPAR/TQGPAR order.
        interaction_index: usize,
        /// One-based TQGPAR expression row.
        expression_index: usize,
        /// One-based coefficient term.
        term_index: usize,
    },
    /// One magnetic interaction value.
    Magnetic {
        /// One-based phase index.
        phase_index: usize,
        /// One-based interaction index in TQLPAR/TQGPAR order.
        interaction_index: usize,
        /// One-based TQGPAR expression row.
        expression_index: usize,
        /// The manual-defined magnetic column.
        role: MagneticInteractionRole,
    },
}

impl InteractionParameterAddress {
    fn checked_parts(self) -> Result<(usize, usize, usize, usize, usize), ChemAppError> {
        let (i1, interaction, expression, term, phase) = match self {
            Self::Gibbs {
                phase_index,
                interaction_index,
                expression_index,
                term_index,
            } => (
                13,
                interaction_index,
                expression_index,
                term_index,
                phase_index,
            ),
            Self::Magnetic {
                phase_index,
                interaction_index,
                expression_index,
                role,
            } => (
                10,
                interaction_index,
                expression_index,
                role.tqcdat_selector(),
                phase_index,
            ),
        };
        if interaction == 0 || expression == 0 || term == 0 || phase == 0 {
            return Err(ChemAppError::OtherError(
                "interaction parameter addresses use one-based nonzero indices".to_owned(),
            ));
        }
        Ok((i1, interaction, expression, term, phase))
    }

    /// Return the exact five integer selectors passed to raw TQCDAT.
    pub fn tqcdat_selectors(self) -> Result<[usize; 5], ChemAppError> {
        let (i1, i2, i3, i4, i5) = self.checked_parts()?;
        Ok([i1, i2, i3, i4, i5])
    }

    /// Return the TQGPAR channel for readback.
    pub fn channel(self) -> InteractionChannel {
        match self {
            Self::Gibbs { .. } => InteractionChannel::GibbsExcess,
            Self::Magnetic { .. } => InteractionChannel::Magnetic,
        }
    }

    fn matrix_coordinates(self) -> (usize, usize, usize, usize) {
        match self {
            Self::Gibbs {
                phase_index,
                interaction_index,
                expression_index,
                term_index,
            } => (phase_index, interaction_index, expression_index, term_index),
            Self::Magnetic {
                phase_index,
                interaction_index,
                expression_index,
                role,
            } => (
                phase_index,
                interaction_index,
                expression_index,
                role.tqcdat_selector(),
            ),
        }
    }
}

/// One addressable live interaction value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InteractionParameter {
    /// Structural mutation/readback address.
    pub address: InteractionParameterAddress,
    /// Value returned by live TQGPAR in the active default units.
    pub value: f64,
}

/// Semantic role of a returned TQGPAR cell.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InteractionParameterRole {
    /// An ordinary Gibbs coefficient identified by one-based term number.
    GibbsTerm { term_index: usize },
    /// Excess Curie/Neel temperature.
    CurieNeelTemperature,
    /// Excess magnetic moment.
    MagneticMoment,
    /// A returned column whose mutation selector is not yet established.
    Unclassified { column_index: usize },
}

impl Display for InteractionParameterRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GibbsTerm { term_index } => write!(formatter, "Gibbs term {term_index}"),
            Self::CurieNeelTemperature => formatter.write_str("Curie/Neel temperature"),
            Self::MagneticMoment => formatter.write_str("Magnetic moment"),
            Self::Unclassified { column_index } => {
                write!(formatter, "Unclassified column {column_index}")
            }
        }
    }
}

/// Whether one returned TQGPAR cell has a verified TQCDAT mutation address.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InteractionMutationSupport {
    /// The contained address was verified by mutate/read/restore round-trip.
    Verified(InteractionParameterAddress),
    /// The value remains inspectable, but mutation is deliberately unavailable.
    ReadOnly { reason: String },
}

/// One cell of the complete logical TQGPAR matrix.
#[derive(Clone, Debug, PartialEq)]
pub struct InteractionParameterCell {
    /// One-based expression row.
    pub expression_index: usize,
    /// One-based returned TQGPAR column.
    pub column_index: usize,
    /// Model/channel-aware meaning of the column.
    pub role: InteractionParameterRole,
    /// Live value returned by TQGPAR.
    pub value: f64,
    /// Verified address or an explicit read-only reason.
    pub mutation: InteractionMutationSupport,
}

/// Provenance of the structural descriptor used for name resolution.
///
/// The exact TQLPAR text is retained in [`InteractionRaw`] for both variants.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InteractionDescriptorSource {
    /// The parsed native TQLPAR descriptor was used unchanged.
    Native,
    /// An optional ASCII-DAT provider supplied the structural descriptor.
    DatRecovered,
}

impl Display for InteractionDescriptorSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Native => "Native",
            Self::DatRecovered => "DAT recovered",
        })
    }
}

/// Positively classified reason that DAT structure replaced native structure.
///
/// This is deliberately typed: a mere cross-source difference is not a
/// recovery condition and cannot select the DAT descriptor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InteractionRecoveryReason {
    /// The native descriptor differs only where TQLPAR printed `[*]` and the
    /// deterministically corresponding DAT interaction has an order >= 10.
    KnownMultiDigitOrderCorruption,
    /// Reserved for a positively identified malformed-output defect. Merely
    /// failing the current Rust grammar never selects this recovery.
    MalformedNativeDescriptor,
    /// Reserved for another independently validated native defect class.
    OtherValidatedNativeDefect,
}

impl Display for InteractionRecoveryReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::KnownMultiDigitOrderCorruption => "known multi-digit order corruption",
            Self::MalformedNativeDescriptor => "malformed native descriptor",
            Self::OtherValidatedNativeDefect => "other validated native defect",
        })
    }
}

/// Result of independently comparing native TQLPAR structure with ASCII-DAT
/// structure supplied by an optional provider.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InteractionCrossCheck {
    /// No provider was requested; the interaction is native-only.
    NotRequested,
    /// The provider has no corresponding DAT descriptor for this row.
    Unavailable,
    /// Native and DAT typed structures agree exactly.
    Agree,
    /// The provider failed for this row. Native parsing/resolution is retained.
    DatError {
        /// Row-local diagnostic from the provider or its contract validation.
        reason: String,
    },
    /// Both structures are retained, but native remains effective because no
    /// validated native defect explains the difference.
    Disagree {
        /// Independent DAT-side structural evidence.
        dat_descriptor: InteractionDescriptor,
        /// Why the difference was not promoted to a recovery.
        reason: String,
    },
    /// DAT becomes effective only for this positively classified native defect.
    ValidatedRecovery {
        /// Independent DAT-side structure selected for effective resolution.
        dat_descriptor: InteractionDescriptor,
        /// Typed rule authorizing DAT selection.
        reason: InteractionRecoveryReason,
    },
}

impl Display for InteractionCrossCheck {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRequested => formatter.write_str("Not requested"),
            Self::Unavailable => formatter.write_str("DAT unavailable"),
            Self::Agree => formatter.write_str("Agree"),
            Self::DatError { reason } => write!(formatter, "DAT error: {reason}"),
            Self::Disagree { reason, .. } => write!(formatter, "Disagree: {reason}"),
            Self::ValidatedRecovery { reason, .. } => {
                write!(formatter, "DAT recovered: {reason}")
            }
        }
    }
}

/// An opaque one-based index as printed by TQLPAR.
///
/// Its namespace is model-dependent and is intentionally not called a phase
/// constituent or sublattice species until resolution has consulted TQMODL.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NativeInteractionIndex(pub usize);

/// An exponent/order token attached to a native interaction member.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InteractionOrder {
    /// A numeric order such as `[0]` or `[3]`.
    Numeric(usize),
    /// ChemApp's literal wildcard/order token `[*]`.
    Wildcard,
}

impl Display for InteractionOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Numeric(value) => write!(f, "{value}"),
            Self::Wildcard => f.write_str("*"),
        }
    }
}

/// One indexed member with its preserved order token.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NativePoweredMember {
    /// The one-based model-dependent member index printed by TQLPAR.
    pub index: NativeInteractionIndex,
    /// The numeric or wildcard order printed for this member.
    pub order: InteractionOrder,
}

/// The structurally parsed form of a TQLPAR descriptor.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum InteractionDescriptor {
    /// A powered binary/ternary member list forming the first group, optionally
    /// followed by further colon-separated sublattice groups.
    Powered {
        /// The one-based parameter index printed at the start of the record.
        parameter_index: usize,
        /// The arity printed after the native `*` marker.
        declared_arity: usize,
        /// Ordered indexed members and exponent/order tokens in sublattice 1.
        first_sublattice: Vec<NativePoweredMember>,
        /// Additional sublattice groups following `:`. Group boundaries are
        /// structural and are never flattened into the powered member list.
        following_sublattices: Vec<Vec<NativeInteractionIndex>>,
        /// Native interaction-family label, retained without a closed enum.
        type_label: Option<String>,
    },
    /// Colon-separated sublattice groups. Each group contains one or more
    /// flattened sublattice indices.
    SublatticeGroups {
        /// The one-based parameter index printed at the start of the record.
        parameter_index: usize,
        /// The arity printed after the native `*` marker.
        declared_arity: usize,
        /// Ordered colon-separated groups of native member indices.
        sublattices: Vec<Vec<NativeInteractionIndex>>,
        /// Optional native interaction-family label.
        type_label: Option<String>,
    },
    /// A reciprocal interaction whose two sides remain distinct.
    Reciprocal {
        /// The one-based parameter index printed at the start of the record.
        parameter_index: usize,
        /// Ordered first side of the reciprocal interaction.
        left: [NativeInteractionIndex; 2],
        /// Ordered second side of the reciprocal interaction.
        right: [NativeInteractionIndex; 2],
        /// Optional native interaction-family label.
        type_label: Option<String>,
    },
    /// Syntax not understood by the current grammar. The complete native text
    /// remains available for forward-compatible diagnostics.
    Unparsed {
        /// Exact descriptor returned by TQLPAR.
        raw: String,
        /// Context explaining why the current structural grammar rejected it.
        reason: String,
    },
}

impl InteractionDescriptor {
    /// A stable human-readable structural family name.
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Powered { .. } => "Powered",
            Self::SublatticeGroups { .. } => "Sublattice groups",
            Self::Reciprocal { .. } => "Reciprocal",
            Self::Unparsed { .. } => "Unparsed",
        }
    }

    /// Number of sublattice groups represented by the descriptor.
    ///
    /// This is structural: the native `*N` marker is an interaction arity,
    /// not a sublattice count. A descriptor with `S` sublattices contains
    /// exactly `S - 1` colon separators. Unparsed syntax has no trusted count.
    pub fn sublattice_count(&self) -> Option<usize> {
        match self {
            Self::Powered {
                following_sublattices,
                ..
            } => Some(1 + following_sublattices.len()),
            Self::SublatticeGroups { sublattices, .. } => Some(sublattices.len()),
            Self::Reciprocal { .. } => Some(2),
            Self::Unparsed { .. } => None,
        }
    }

    fn parameter_index(&self) -> Option<usize> {
        match self {
            Self::Powered {
                parameter_index, ..
            }
            | Self::SublatticeGroups {
                parameter_index, ..
            }
            | Self::Reciprocal {
                parameter_index, ..
            } => Some(*parameter_index),
            Self::Unparsed { .. } => None,
        }
    }
}

/// Context supplied to an optional ASCII-DAT descriptor cross-check provider.
///
/// Providers should map the one-based TQLPAR/TQGPAR parameter index to a DAT
/// semantic interaction deterministically. Runtime agreement on healthy rows
/// is evidence for that mapping, not an unconditional cross-model ordering law.
#[derive(Clone, Copy, Debug)]
pub struct InteractionCrossCheckRequest<'a> {
    /// One-based ChemApp phase index.
    pub phase_index: usize,
    /// Phase name returned by TQGNP.
    pub phase_name: &'a str,
    /// Solution-model code returned by TQMODL.
    pub model: &'a str,
    /// Independently queried Gibbs or magnetic channel.
    pub channel: InteractionChannel,
    /// One-based TQLPAR/TQGPAR interaction index.
    pub parameter_index: usize,
    /// Exact native TQLPAR text, including any corruption.
    pub native_descriptor: &'a str,
    /// Best-effort parse. Valid-looking output may still be wrong when ChemApp
    /// scrambles a multi-digit order into `[*]`.
    pub native_parsed: &'a InteractionDescriptor,
}

/// Optional, dependency-neutral source of compatible ASCII-DAT interaction
/// structure for cross-checking.
///
/// The base crate remains usable with CST/BIN inputs and has no mandatory DAT
/// parser dependency. The provider supplies evidence for the exact
/// phase/channel/index or `None` when unavailable; it does not decide which
/// descriptor is authoritative. It cannot supply numerical parameters: live
/// TQGPAR output remains authoritative. Errors are row-local diagnostics and
/// never invalidate an otherwise usable native descriptor.
pub trait InteractionDescriptorCrossCheck {
    /// Return the deterministic DAT-side descriptor corresponding to this row.
    fn descriptor_for(
        &self,
        request: InteractionCrossCheckRequest<'_>,
    ) -> Result<Option<InteractionDescriptor>, String>;
}

/// Compatibility name for the former recovery-oriented trait. This aliases
/// the single cross-check contract rather than preserving competing semantics.
#[deprecated(note = "use InteractionDescriptorCrossCheck")]
pub use InteractionDescriptorCrossCheck as InteractionDescriptorRecovery;

/// Compatibility name for the former recovery request.
#[deprecated(note = "use InteractionCrossCheckRequest")]
pub use InteractionCrossCheckRequest as InteractionRecoveryRequest;

/// An owned native interaction record preserving exact textual evidence and
/// authoritative live numerical values.
#[derive(Clone, Debug, PartialEq)]
pub struct InteractionRaw {
    /// One-based ChemApp phase index.
    pub phase_index: usize,
    /// Phase name obtained from TQGNP.
    pub phase_name: String,
    /// Solution-model code obtained from TQMODL.
    pub model: String,
    /// Number of sublattices returned by TQNOSL for the containing phase.
    pub sublattice_count: usize,
    /// Independently queried Gibbs or magnetic channel.
    pub channel: InteractionChannel,
    /// One-based TQLPAR/TQGPAR parameter index.
    pub parameter_index: usize,
    /// Exact descriptor returned by TQLPAR.
    pub raw_descriptor: String,
    /// TQGPAR's logical `NOEXPR` rows by `NVALA` columns.
    pub values: Vec<Vec<f64>>,
}

/// A member identity resolved through native ChemApp metadata.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum InteractionMember {
    /// A TQGNPC phase-constituent identity.
    PhaseConstituent {
        /// One-based phase-constituent index.
        index: usize,
        /// Name returned by TQGNPC.
        name: String,
    },
    /// A TQGNLC sublattice-local identity. `encoded_index` is retained only
    /// for comparison with the native flattened descriptor.
    SublatticeSpecies {
        /// One-based index in the descriptor's flattened namespace.
        encoded_index: usize,
        /// One-based sublattice number.
        sublattice: usize,
        /// One-based constituent index local to the sublattice.
        local_index: usize,
        /// Name returned by TQGNLC.
        name: String,
    },
}

impl Display for InteractionMember {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PhaseConstituent { index, name } => write!(f, "{name} [P:{index}]"),
            Self::SublatticeSpecies {
                sublattice,
                local_index,
                name,
                ..
            } => write!(f, "{name} [S{sublattice}:{local_index}]"),
        }
    }
}

impl InteractionMember {
    /// Return the native ChemApp member name without diagnostic index
    /// annotations. Structural identity remains available in the enum fields.
    fn name(&self) -> &str {
        match self {
            Self::PhaseConstituent { name, .. } | Self::SublatticeSpecies { name, .. } => name,
        }
    }
}

/// A resolved member and its preserved exponent/order.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ResolvedPoweredMember {
    /// Model-aware identity resolved from native ChemApp metadata.
    pub member: InteractionMember,
    /// Preserved exponent/order token.
    pub order: InteractionOrder,
}

/// Name-resolved interaction structure. It mirrors the parsed grouping so
/// ordering, reciprocal sides, powers, and type labels remain inspectable.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ResolvedInteractionDescriptor {
    /// Name-resolved powered first group and following sublattice groups.
    Powered {
        /// Arity printed by the native descriptor.
        declared_arity: usize,
        /// Ordered name-resolved members in sublattice 1.
        first_sublattice: Vec<ResolvedPoweredMember>,
        /// Name-resolved sublattice groups following `:`.
        following_sublattices: Vec<Vec<InteractionMember>>,
        /// Optional native interaction-family label.
        type_label: Option<String>,
    },
    /// Name-resolved colon-separated sublattice groups.
    SublatticeGroups {
        /// Arity printed by the native descriptor.
        declared_arity: usize,
        /// Ordered resolved groups; group boundaries remain significant.
        sublattices: Vec<Vec<InteractionMember>>,
        /// Optional native interaction-family label.
        type_label: Option<String>,
    },
    /// Name-resolved reciprocal sides, retained as two distinct pairs.
    Reciprocal {
        /// Ordered first reciprocal side.
        left: [InteractionMember; 2],
        /// Ordered second reciprocal side.
        right: [InteractionMember; 2],
        /// Optional native interaction-family label.
        type_label: Option<String>,
    },
}

impl ResolvedInteractionDescriptor {
    /// Number of sublattices retained in this resolved descriptor.
    pub fn sublattice_count(&self) -> usize {
        match self {
            Self::Powered {
                following_sublattices,
                ..
            } => 1 + following_sublattices.len(),
            Self::SublatticeGroups { sublattices, .. } => sublattices.len(),
            Self::Reciprocal { .. } => 2,
        }
    }
}

fn label_suffix(label: &Option<String>) -> String {
    label
        .as_ref()
        .map(|value| format!(" ({value})"))
        .unwrap_or_default()
}

impl Display for ResolvedInteractionDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Powered {
                first_sublattice,
                following_sublattices,
                type_label,
                ..
            } => {
                for (offset, member) in first_sublattice.iter().enumerate() {
                    if offset > 0 {
                        f.write_str("-")?;
                    }
                    write!(f, "({})^[{}]", member.member.name(), member.order)?;
                }
                for sublattice in following_sublattices {
                    f.write_str(" : ")?;
                    for (member_offset, member) in sublattice.iter().enumerate() {
                        if member_offset > 0 {
                            f.write_str("-")?;
                        }
                        write!(f, "({})", member.name())?;
                    }
                }
                f.write_str(&label_suffix(type_label))
            }
            Self::SublatticeGroups {
                sublattices,
                type_label,
                ..
            } => {
                for (sublattice_offset, sublattice) in sublattices.iter().enumerate() {
                    if sublattice_offset > 0 {
                        f.write_str(" : ")?;
                    }
                    for (member_offset, member) in sublattice.iter().enumerate() {
                        if member_offset > 0 {
                            f.write_str("-")?;
                        }
                        write!(f, "({})", member.name())?;
                    }
                }
                f.write_str(&label_suffix(type_label))
            }
            Self::Reciprocal {
                left,
                right,
                type_label,
            } => write!(
                f,
                "({})-({}) : ({})-({}){}",
                left[0].name(),
                left[1].name(),
                right[0].name(),
                right[1].name(),
                label_suffix(type_label)
            ),
        }
    }
}

/// Resolution outcome kept alongside every raw row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InteractionResolution {
    /// Model-aware name resolution succeeded.
    Resolved(ResolvedInteractionDescriptor),
    /// Parsing or model-aware index resolution could not be completed.
    Unresolved {
        /// Context retained for diagnostics rather than dropping the row.
        reason: String,
    },
}

/// One complete native → parsed → cross-checked → effective → resolved row.
#[derive(Clone, Debug, PartialEq)]
pub struct Interaction {
    /// Authoritative native descriptor, identity, and coefficient matrix.
    pub raw: InteractionRaw,
    /// Structural interpretation of the original TQLPAR text. This is never
    /// overwritten by DAT recovery, even when the native text is malformed.
    pub native_parsed: InteractionDescriptor,
    /// Independent DAT comparison status and any retained DAT-side structure.
    pub cross_check: InteractionCrossCheck,
    /// Descriptor actually used for model-aware name resolution. It equals
    /// `native_parsed` except for a typed validated-recovery condition.
    pub effective_descriptor: InteractionDescriptor,
    /// Model-aware resolution of `effective_descriptor`.
    pub resolution: InteractionResolution,
}

impl Interaction {
    /// True only when both parsing and model-aware name resolution succeeded.
    pub fn is_resolved(&self) -> bool {
        matches!(self.resolution, InteractionResolution::Resolved(_))
    }

    /// Origin of the descriptor used for name resolution.
    pub fn effective_source(&self) -> InteractionDescriptorSource {
        if matches!(
            &self.cross_check,
            InteractionCrossCheck::ValidatedRecovery { .. }
        ) {
            InteractionDescriptorSource::DatRecovered
        } else {
            InteractionDescriptorSource::Native
        }
    }

    /// Render the deterministic name-based form or an explicit unresolved
    /// diagnostic; the raw descriptor is always available separately.
    pub fn resolved_text(&self) -> String {
        match &self.resolution {
            InteractionResolution::Resolved(value) => value.to_string(),
            InteractionResolution::Unresolved { reason } => format!("UNRESOLVED: {reason}"),
        }
    }

    /// Expand the complete logical `NOEXPR × NVALA` matrix into typed cells.
    ///
    /// EN22 Win64 round-trips establish ordinary six-term Gibbs addresses for
    /// SUBQ/SUBL/SUBLM/QKTO/QKTOM and both magnetic roles for SUBLM. SUBQ's
    /// returned columns 7–18 are retained but read-only: the generic Gibbs
    /// selector was rejected by ChemApp, and their special meaning must not be
    /// guessed. SUBG is likewise read-only pending dedicated runtime evidence.
    pub fn parameter_cells(&self) -> Vec<InteractionParameterCell> {
        let model = self.raw.model.trim().to_ascii_uppercase();
        let mut cells = Vec::new();
        for (expression_offset, row) in self.raw.values.iter().enumerate() {
            for (column_offset, value) in row.iter().copied().enumerate() {
                let expression_index = expression_offset + 1;
                let column_index = column_offset + 1;
                let (role, mutation) = match self.raw.channel {
                    InteractionChannel::Magnetic => match column_index {
                        1 => {
                            let address = InteractionParameterAddress::Magnetic {
                                phase_index: self.raw.phase_index,
                                interaction_index: self.raw.parameter_index,
                                expression_index,
                                role: MagneticInteractionRole::CurieNeelTemperature,
                            };
                            (
                                InteractionParameterRole::CurieNeelTemperature,
                                if model == "SUBLM" {
                                    InteractionMutationSupport::Verified(address)
                                } else {
                                    InteractionMutationSupport::ReadOnly {
                                        reason: format!(
                                            "magnetic mutation is not runtime-verified for model {model}"
                                        ),
                                    }
                                },
                            )
                        }
                        2 => {
                            let address = InteractionParameterAddress::Magnetic {
                                phase_index: self.raw.phase_index,
                                interaction_index: self.raw.parameter_index,
                                expression_index,
                                role: MagneticInteractionRole::MagneticMoment,
                            };
                            (
                                InteractionParameterRole::MagneticMoment,
                                if model == "SUBLM" {
                                    InteractionMutationSupport::Verified(address)
                                } else {
                                    InteractionMutationSupport::ReadOnly {
                                        reason: format!(
                                            "magnetic mutation is not runtime-verified for model {model}"
                                        ),
                                    }
                                },
                            )
                        }
                        _ => (
                            InteractionParameterRole::Unclassified { column_index },
                            InteractionMutationSupport::ReadOnly {
                                reason: "the manual defines exactly two magnetic columns"
                                    .to_owned(),
                            },
                        ),
                    },
                    InteractionChannel::GibbsExcess => {
                        let role = if column_index <= 6 {
                            InteractionParameterRole::GibbsTerm {
                                term_index: column_index,
                            }
                        } else {
                            InteractionParameterRole::Unclassified { column_index }
                        };
                        let verified_model =
                            matches!(model.as_str(), "SUBQ" | "SUBL" | "SUBLM" | "QKTO" | "QKTOM");
                        let mutation = if verified_model && column_index <= 6 {
                            InteractionMutationSupport::Verified(
                                InteractionParameterAddress::Gibbs {
                                    phase_index: self.raw.phase_index,
                                    interaction_index: self.raw.parameter_index,
                                    expression_index,
                                    term_index: column_index,
                                },
                            )
                        } else if model == "SUBQ" && column_index > 6 {
                            InteractionMutationSupport::ReadOnly {
                                reason: "SUBQ extended column rejected the documented generic Gibbs selector in the checked runtime".to_owned(),
                            }
                        } else if model == "SUBG" {
                            InteractionMutationSupport::ReadOnly {
                                reason:
                                    "SUBG term/power mutation is documented but runtime-unverified"
                                        .to_owned(),
                            }
                        } else {
                            InteractionMutationSupport::ReadOnly {
                                reason: format!(
                                    "Gibbs mutation is not runtime-verified for model {model} column {column_index}"
                                ),
                            }
                        };
                        (role, mutation)
                    }
                };
                cells.push(InteractionParameterCell {
                    expression_index,
                    column_index,
                    role,
                    value,
                    mutation,
                });
            }
        }
        cells
    }

    /// Render this interaction's complete parameter matrix and mutation status.
    ///
    /// The normal interaction inventory stays descriptor-focused; this
    /// separate table exposes one row per expression/column for debugging and
    /// sensitivity workflows.
    pub fn parameter_table_string(&self) -> String {
        let rows = self
            .parameter_cells()
            .into_iter()
            .map(|cell| {
                let support = match cell.mutation {
                    InteractionMutationSupport::Verified(address) => address
                        .tqcdat_selectors()
                        .map(|selectors| format!("Verified TQCDAT{selectors:?}"))
                        .unwrap_or_else(|error| format!("Invalid: {error}")),
                    InteractionMutationSupport::ReadOnly { reason } => {
                        format!("Read-only: {reason}")
                    }
                };
                vec![
                    cell.expression_index.to_string(),
                    cell.column_index.to_string(),
                    cell.role.to_string(),
                    format!("{:.8e}", cell.value),
                    support,
                ]
            })
            .collect();
        crate::table::render(
            &format!(
                "{} / {} interaction {} parameters",
                self.raw.phase_name, self.raw.channel, self.raw.parameter_index
            ),
            &["Expression", "Column", "Role", "Current value", "Mutation"].map(str::to_owned),
            rows,
        )
    }
}

pub(crate) fn read_interaction_parameter(
    engine: &Engine,
    address: InteractionParameterAddress,
) -> Result<InteractionParameter, ChemAppError> {
    address.checked_parts()?;
    let (phase, interaction, expression, column) = address.matrix_coordinates();
    let values = engine.tqgpar(phase, address.channel().option(), interaction)?;
    let value = values
        .get(expression - 1)
        .and_then(|row| row.get(column - 1))
        .copied()
        .ok_or_else(|| {
            ChemAppError::OtherError(format!(
                "TQGPAR matrix has no expression {expression}, column {column} for phase {phase}, interaction {interaction}"
            ))
        })?;
    Ok(InteractionParameter { address, value })
}

pub(crate) fn write_interaction_parameter(
    engine: &Engine,
    address: InteractionParameterAddress,
    value: f64,
) -> Result<(), ChemAppError> {
    let [i1, i2, i3, i4, i5] = address.tqcdat_selectors()?;
    engine.tqcdat(i1, i2, i3, i4, i5, value)
}

pub(crate) fn validate_interaction_parameter_mutation(
    engine: &Engine,
    address: InteractionParameterAddress,
) -> Result<(), ChemAppError> {
    address.checked_parts()?;
    // A successful read proves the requested expression/column exists before
    // the state-changing call. Model/channel policy then restricts the safe
    // API to the families covered by the Win64 EN22 round-trip audit.
    read_interaction_parameter(engine, address)?;
    let (phase_index, _, _, column_index) = address.matrix_coordinates();
    let model = engine.tqmodl(phase_index)?.trim().to_ascii_uppercase();
    let supported = match address {
        InteractionParameterAddress::Gibbs { .. } => {
            column_index <= 6
                && matches!(model.as_str(), "SUBQ" | "SUBL" | "SUBLM" | "QKTO" | "QKTOM")
        }
        InteractionParameterAddress::Magnetic { .. } => model == "SUBLM",
    };
    if supported {
        Ok(())
    } else {
        Err(ChemAppError::OtherError(format!(
            "interaction mutation is not runtime-verified for model {model}, channel {}, column {column_index}",
            address.channel()
        )))
    }
}

/// Both interaction channels for one solution phase.
#[derive(Clone, Debug, PartialEq)]
pub struct PhaseInteractionReport {
    /// One-based ChemApp phase index.
    pub phase_index: usize,
    /// Phase name returned by TQGNP.
    pub phase_name: String,
    /// Solution-model code returned by TQMODL.
    pub model: String,
    /// Authoritative number of sublattices returned by TQNOSL for this phase.
    pub sublattice_count: usize,
    /// Independently retrieved Gibbs excess interactions.
    pub gibbs: Vec<Interaction>,
    /// Independently retrieved magnetic interactions.
    pub magnetic: Vec<Interaction>,
}

impl PhaseInteractionReport {
    /// Render raw descriptors, resolved names, values, and resolution state in
    /// the shared `comfy-table` style.
    pub fn table_string(&self) -> String {
        let interactions = self.gibbs.iter().chain(&self.magnetic);
        let rows = interactions
            .map(|interaction| {
                vec![
                    format!("{} [{}]", self.phase_name, self.phase_index),
                    self.model.clone(),
                    interaction.raw.sublattice_count.to_string(),
                    interaction.raw.channel.to_string(),
                    interaction.raw.parameter_index.to_string(),
                    interaction.effective_descriptor.kind_name().to_owned(),
                    interaction.effective_source().to_string(),
                    interaction.cross_check.to_string(),
                    interaction.raw.raw_descriptor.clone(),
                    interaction.resolved_text(),
                    format_values(&interaction.raw.values),
                    if interaction.is_resolved() {
                        "Resolved".to_owned()
                    } else {
                        "Unresolved".to_owned()
                    },
                ]
            })
            .collect();
        crate::table::render(
            "ChemApp interactions",
            &[
                "Phase",
                "Model",
                "Sublattices",
                "Channel",
                "Index",
                "Kind",
                "Effective source",
                "Cross-check",
                "Native/indexed",
                "Name-based",
                "Values",
                "State",
            ]
            .map(str::to_owned),
            rows,
        )
    }
}

fn format_values(values: &[Vec<f64>]) -> String {
    let mut output = String::from("[");
    for (row_offset, row) in values.iter().enumerate() {
        if row_offset > 0 {
            output.push_str(", ");
        }
        output.push('[');
        for (column_offset, value) in row.iter().enumerate() {
            if column_offset > 0 {
                output.push_str(", ");
            }
            let _ = write!(output, "{value:.8e}");
        }
        output.push(']');
    }
    output.push(']');
    output
}

fn parse_parenthesized_index(token: &str) -> Result<NativeInteractionIndex, String> {
    let token = token.trim();
    if !token.starts_with('(') || !token.ends_with(')') {
        return Err(format!("expected parenthesized index, found {token:?}"));
    }
    let value = token[1..token.len() - 1]
        .trim()
        .parse::<usize>()
        .map_err(|_| format!("invalid interaction index in {token:?}"))?;
    if value == 0 {
        return Err("interaction indices are one-based".to_owned());
    }
    Ok(NativeInteractionIndex(value))
}

fn parse_powered_member(token: &str) -> Result<NativePoweredMember, String> {
    let (member, order) = token
        .trim()
        .split_once("^[")
        .ok_or_else(|| format!("missing power token in {token:?}"))?;
    if !order.ends_with(']') {
        return Err(format!("unterminated power token in {token:?}"));
    }
    let order = match order[..order.len() - 1].trim() {
        "*" => InteractionOrder::Wildcard,
        value => InteractionOrder::Numeric(
            value
                .parse::<usize>()
                .map_err(|_| format!("invalid power {value:?}"))?,
        ),
    };
    Ok(NativePoweredMember {
        index: parse_parenthesized_index(member)?,
        order,
    })
}

fn split_type_label(body: &str) -> (&str, Option<String>) {
    let trimmed = body.trim_end();
    if !trimmed.ends_with(')') {
        return (trimmed, None);
    }
    if let Some(start) = trimmed.rfind(" (") {
        let candidate = &trimmed[start + 2..trimmed.len() - 1];
        if candidate.chars().any(char::is_alphabetic) {
            return (trimmed[..start].trim_end(), Some(candidate.to_owned()));
        }
    }
    (trimmed, None)
}

fn parse_group(group: &str) -> Result<Vec<NativeInteractionIndex>, String> {
    let members: Result<Vec<_>, _> = group.split('-').map(parse_parenthesized_index).collect();
    let members = members?;
    if members.is_empty() {
        Err("empty interaction group".to_owned())
    } else {
        Ok(members)
    }
}

fn parse_descriptor_result(raw: &str) -> Result<InteractionDescriptor, String> {
    let (index, remainder) = raw
        .split_once(':')
        .ok_or_else(|| "missing parameter-index separator".to_owned())?;
    let parameter_index = index
        .trim()
        .parse::<usize>()
        .map_err(|_| format!("invalid parameter index {index:?}"))?;
    let remainder = remainder.trim_start();
    let remainder = remainder
        .strip_prefix('*')
        .ok_or_else(|| "missing interaction arity marker".to_owned())?;

    if let Some(body) = remainder.strip_prefix('R') {
        let (body, type_label) = split_type_label(body.trim_start());
        let groups: Result<Vec<_>, _> = body.split(':').map(parse_group).collect();
        let groups = groups?;
        if groups.len() != 2 || groups[0].len() != 2 || groups[1].len() != 2 {
            return Err("reciprocal interaction requires two two-member sides".to_owned());
        }
        return Ok(InteractionDescriptor::Reciprocal {
            parameter_index,
            left: [groups[0][0], groups[0][1]],
            right: [groups[1][0], groups[1][1]],
            type_label,
        });
    }

    let arity_end = remainder
        .find(char::is_whitespace)
        .ok_or_else(|| "missing interaction body".to_owned())?;
    let declared_arity = remainder[..arity_end]
        .parse::<usize>()
        .map_err(|_| "invalid interaction arity".to_owned())?;
    let (body, type_label) = split_type_label(remainder[arity_end..].trim_start());

    if body.contains("^[") {
        let groups: Vec<_> = body.split(':').map(str::trim).collect();
        let first_sublattice: Result<Vec<_>, _> =
            groups[0].split('-').map(parse_powered_member).collect();
        let first_sublattice = first_sublattice?;
        if first_sublattice.is_empty() {
            return Err("powered interaction has no members".to_owned());
        }
        let following_sublattices = groups[1..]
            .iter()
            .map(|group| parse_group(group))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(InteractionDescriptor::Powered {
            parameter_index,
            declared_arity,
            first_sublattice,
            following_sublattices,
            type_label,
        })
    } else {
        let sublattices: Result<Vec<_>, _> = body.split(':').map(parse_group).collect();
        let sublattices = sublattices?;
        if sublattices.is_empty() {
            return Err("interaction has no groups".to_owned());
        }
        Ok(InteractionDescriptor::SublatticeGroups {
            parameter_index,
            declared_arity,
            sublattices,
            type_label,
        })
    }
}

/// Parse a complete TQLPAR descriptor. Unrecognized syntax is returned as an
/// explicit [`InteractionDescriptor::Unparsed`] value with the raw text.
pub fn parse_interaction_descriptor(raw: &str) -> InteractionDescriptor {
    parse_descriptor_result(raw).unwrap_or_else(|reason| InteractionDescriptor::Unparsed {
        raw: raw.to_owned(),
        reason,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResolutionStrategy {
    PhaseConstituents,
    FlattenedSublattices,
    Unsupported,
}

fn resolution_strategy(model: &str) -> ResolutionStrategy {
    match model.trim().to_ascii_uppercase().as_str() {
        "QKTO" | "QKTOM" => ResolutionStrategy::PhaseConstituents,
        "SUBL" | "SUBLM" | "SUBQ" | "SUBQM" => ResolutionStrategy::FlattenedSublattices,
        _ => ResolutionStrategy::Unsupported,
    }
}

#[derive(Debug)]
struct InteractionMetadata {
    phase_constituents: Vec<String>,
    sublattices: Vec<Vec<String>>,
}

impl InteractionMetadata {
    fn from_engine(engine: &Engine, phase_index: usize) -> Result<Self, ChemAppError> {
        let phase_constituents = (1..=engine.tqnopc(phase_index)?)
            .map(|index| engine.tqgnpc(phase_index, index))
            .collect::<Result<Vec<_>, _>>()?;
        let sublattices = (1..=engine.tqnosl(phase_index)?)
            .map(|sublattice| {
                (1..=engine.tqnolc(phase_index, sublattice)?)
                    .map(|index| engine.tqgnlc(phase_index, sublattice, index))
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            phase_constituents,
            sublattices,
        })
    }
}

fn resolve_flattened(
    index: NativeInteractionIndex,
    sublattices: &[Vec<String>],
) -> Result<InteractionMember, String> {
    if index.0 == 0 {
        return Err("flattened sublattice index zero is invalid".to_owned());
    }
    let mut first = 1usize;
    for (sublattice_offset, names) in sublattices.iter().enumerate() {
        let last = first + names.len();
        if index.0 >= first && index.0 < last {
            let local_index = index.0 - first + 1;
            return Ok(InteractionMember::SublatticeSpecies {
                encoded_index: index.0,
                sublattice: sublattice_offset + 1,
                local_index,
                name: names[local_index - 1].clone(),
            });
        }
        first = last;
    }
    Err(format!(
        "flattened sublattice index {} exceeds the {} known species",
        index.0,
        first.saturating_sub(1)
    ))
}

fn resolve_member(
    index: NativeInteractionIndex,
    strategy: ResolutionStrategy,
    metadata: &InteractionMetadata,
) -> Result<InteractionMember, String> {
    match strategy {
        ResolutionStrategy::PhaseConstituents => {
            let offset = index
                .0
                .checked_sub(1)
                .ok_or_else(|| "phase-constituent index zero is invalid".to_owned())?;
            metadata
                .phase_constituents
                .get(offset)
                .cloned()
                .map(|name| InteractionMember::PhaseConstituent {
                    index: index.0,
                    name,
                })
                .ok_or_else(|| format!("phase-constituent index {} is out of range", index.0))
        }
        ResolutionStrategy::FlattenedSublattices => resolve_flattened(index, &metadata.sublattices),
        ResolutionStrategy::Unsupported => {
            Err("model has no verified interaction index strategy".to_owned())
        }
    }
}

fn resolve_descriptor(
    descriptor: &InteractionDescriptor,
    strategy: ResolutionStrategy,
    metadata: &InteractionMetadata,
) -> Result<ResolvedInteractionDescriptor, String> {
    if let Some(descriptor_count) = descriptor.sublattice_count() {
        let phase_count = metadata.sublattices.len();
        if descriptor_count != phase_count {
            return Err(format!(
                "descriptor represents {descriptor_count} sublattices, but TQNOSL reports {phase_count} for the phase"
            ));
        }
    }
    match descriptor {
        InteractionDescriptor::Powered {
            declared_arity,
            first_sublattice,
            following_sublattices,
            type_label,
            ..
        } => Ok(ResolvedInteractionDescriptor::Powered {
            declared_arity: *declared_arity,
            first_sublattice: first_sublattice
                .iter()
                .map(|member| {
                    Ok(ResolvedPoweredMember {
                        member: resolve_member(member.index, strategy, metadata)?,
                        order: member.order,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
            following_sublattices: following_sublattices
                .iter()
                .map(|group| {
                    group
                        .iter()
                        .map(|index| resolve_member(*index, strategy, metadata))
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()?,
            type_label: type_label.clone(),
        }),
        InteractionDescriptor::SublatticeGroups {
            declared_arity,
            sublattices,
            type_label,
            ..
        } => Ok(ResolvedInteractionDescriptor::SublatticeGroups {
            declared_arity: *declared_arity,
            sublattices: sublattices
                .iter()
                .map(|group| {
                    group
                        .iter()
                        .map(|index| resolve_member(*index, strategy, metadata))
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()?,
            type_label: type_label.clone(),
        }),
        InteractionDescriptor::Reciprocal {
            left,
            right,
            type_label,
            ..
        } => Ok(ResolvedInteractionDescriptor::Reciprocal {
            left: [
                resolve_member(left[0], strategy, metadata)?,
                resolve_member(left[1], strategy, metadata)?,
            ],
            right: [
                resolve_member(right[0], strategy, metadata)?,
                resolve_member(right[1], strategy, metadata)?,
            ],
            type_label: type_label.clone(),
        }),
        InteractionDescriptor::Unparsed { reason, .. } => Err(reason.clone()),
    }
}

fn is_no_interaction_data(error: &ChemAppError, channel: InteractionChannel) -> bool {
    matches!(
        (channel, error),
        (
            InteractionChannel::GibbsExcess,
            ChemAppError::NativeError(1008)
        ) | (
            InteractionChannel::Magnetic,
            ChemAppError::NativeError(1006 | 1009)
        )
    )
}

/// True only for the validated ChemApp display defect where all structural
/// fields agree and one or more native wildcard orders correspond to DAT
/// numeric orders of at least two digits. A wildcard alone proves nothing.
fn is_known_multi_digit_order_corruption(
    native: &InteractionDescriptor,
    dat: &InteractionDescriptor,
) -> bool {
    let (
        InteractionDescriptor::Powered {
            parameter_index: native_index,
            declared_arity: native_arity,
            first_sublattice: native_members,
            following_sublattices: native_following,
            type_label: native_label,
        },
        InteractionDescriptor::Powered {
            parameter_index: dat_index,
            declared_arity: dat_arity,
            first_sublattice: dat_members,
            following_sublattices: dat_following,
            type_label: dat_label,
        },
    ) = (native, dat)
    else {
        return false;
    };
    if native_index != dat_index
        || native_arity != dat_arity
        || native_following != dat_following
        || native_label != dat_label
        || native_members.len() != dat_members.len()
    {
        return false;
    }

    let mut recovered_order = false;
    for (native_member, dat_member) in native_members.iter().zip(dat_members) {
        if native_member.index != dat_member.index {
            return false;
        }
        match (native_member.order, dat_member.order) {
            (native_order, dat_order) if native_order == dat_order => {}
            (InteractionOrder::Wildcard, InteractionOrder::Numeric(order)) if order >= 10 => {
                recovered_order = true;
            }
            _ => return false,
        }
    }
    recovered_order
}

fn descriptor_with_cross_check(
    raw: &InteractionRaw,
    native_parsed: &InteractionDescriptor,
    cross_check: Option<&dyn InteractionDescriptorCrossCheck>,
    strategy: ResolutionStrategy,
    metadata: &InteractionMetadata,
) -> (InteractionCrossCheck, InteractionDescriptor) {
    let Some(cross_check) = cross_check else {
        return (InteractionCrossCheck::NotRequested, native_parsed.clone());
    };
    let dat_descriptor = match cross_check.descriptor_for(InteractionCrossCheckRequest {
        phase_index: raw.phase_index,
        phase_name: &raw.phase_name,
        model: &raw.model,
        channel: raw.channel,
        parameter_index: raw.parameter_index,
        native_descriptor: &raw.raw_descriptor,
        native_parsed,
    }) {
        Ok(descriptor) => descriptor,
        Err(reason) => {
            return (
                InteractionCrossCheck::DatError { reason },
                native_parsed.clone(),
            )
        }
    };
    let Some(dat_descriptor) = dat_descriptor else {
        return (InteractionCrossCheck::Unavailable, native_parsed.clone());
    };
    if dat_descriptor.parameter_index() != Some(raw.parameter_index) {
        return (
            InteractionCrossCheck::DatError {
                reason: format!(
                    "DAT descriptor parameter index {:?} does not match native row {}",
                    dat_descriptor.parameter_index(),
                    raw.parameter_index
                ),
            },
            native_parsed.clone(),
        );
    }
    if dat_descriptor == *native_parsed {
        return (InteractionCrossCheck::Agree, native_parsed.clone());
    }

    let dat_resolution = resolve_descriptor(&dat_descriptor, strategy, metadata);
    let validated_reason = if dat_resolution.is_ok()
        && is_known_multi_digit_order_corruption(native_parsed, &dat_descriptor)
    {
        Some(InteractionRecoveryReason::KnownMultiDigitOrderCorruption)
    } else {
        None
    };
    if let Some(reason) = validated_reason {
        return (
            InteractionCrossCheck::ValidatedRecovery {
                dat_descriptor: dat_descriptor.clone(),
                reason,
            },
            dat_descriptor,
        );
    }

    let reason = match dat_resolution {
        Ok(_) => "structural difference is not a validated native defect".to_owned(),
        Err(reason) => format!("DAT descriptor is not usable with native metadata: {reason}"),
    };
    (
        InteractionCrossCheck::Disagree {
            dat_descriptor,
            reason,
        },
        native_parsed.clone(),
    )
}

fn interpret_interaction(
    raw: InteractionRaw,
    native_parsed: InteractionDescriptor,
    cross_check_provider: Option<&dyn InteractionDescriptorCrossCheck>,
    strategy: ResolutionStrategy,
    metadata: &InteractionMetadata,
) -> Interaction {
    let native_resolution = resolve_descriptor(&native_parsed, strategy, metadata);
    let (cross_check, effective_descriptor) = descriptor_with_cross_check(
        &raw,
        &native_parsed,
        cross_check_provider,
        strategy,
        metadata,
    );
    let effective_resolution = if matches!(
        &cross_check,
        InteractionCrossCheck::ValidatedRecovery { .. }
    ) {
        resolve_descriptor(&effective_descriptor, strategy, metadata)
    } else {
        native_resolution
    };
    let resolution = effective_resolution
        .map(InteractionResolution::Resolved)
        .unwrap_or_else(|reason| InteractionResolution::Unresolved { reason });
    Interaction {
        raw,
        native_parsed,
        cross_check,
        effective_descriptor,
        resolution,
    }
}

pub(crate) fn load_phase_interactions(
    engine: &Engine,
    phase_index: usize,
    channel: InteractionChannel,
) -> Result<Vec<Interaction>, ChemAppError> {
    load_phase_interactions_with_cross_check(engine, phase_index, channel, None)
}

pub(crate) fn load_phase_interactions_with_cross_check(
    engine: &Engine,
    phase_index: usize,
    channel: InteractionChannel,
    cross_check: Option<&dyn InteractionDescriptorCrossCheck>,
) -> Result<Vec<Interaction>, ChemAppError> {
    let phase_name = engine.tqgnp(phase_index)?;
    let model = engine.tqmodl(phase_index)?;
    if model == "PURE" {
        return Ok(Vec::new());
    }
    let descriptors = match engine.tqlpar(phase_index, channel.option()) {
        Ok(descriptors) => descriptors,
        Err(error) if is_no_interaction_data(&error, channel) => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let metadata = InteractionMetadata::from_engine(engine, phase_index)?;
    let sublattice_count = metadata.sublattices.len();
    let strategy = resolution_strategy(&model);
    descriptors
        .into_iter()
        .enumerate()
        .map(|(offset, raw_descriptor)| {
            let parameter_index = offset + 1;
            let mut parsed = parse_interaction_descriptor(&raw_descriptor);
            if let Some(descriptor_index) = parsed.parameter_index() {
                if descriptor_index != parameter_index {
                    parsed = InteractionDescriptor::Unparsed {
                        raw: raw_descriptor.clone(),
                        reason: format!(
                            "TQLPAR descriptor index {descriptor_index} does not match position {parameter_index}"
                        ),
                    };
                }
            }
            let raw = InteractionRaw {
                phase_index,
                phase_name: phase_name.clone(),
                model: model.clone(),
                sublattice_count,
                channel,
                parameter_index,
                raw_descriptor,
                values: engine.tqgpar(phase_index, channel.option(), parameter_index)?,
            };
            Ok(interpret_interaction(
                raw,
                parsed,
                cross_check,
                strategy,
                &metadata,
            ))
        })
        .collect()
}

pub(crate) fn load_phase_interaction_report_with_cross_check(
    engine: &Engine,
    phase_index: usize,
    cross_check: Option<&dyn InteractionDescriptorCrossCheck>,
) -> Result<PhaseInteractionReport, ChemAppError> {
    let phase_name = engine.tqgnp(phase_index)?;
    let model = engine.tqmodl(phase_index)?;
    let sublattice_count = engine.tqnosl(phase_index)?;
    Ok(PhaseInteractionReport {
        phase_index,
        phase_name,
        model,
        sublattice_count,
        gibbs: load_phase_interactions_with_cross_check(
            engine,
            phase_index,
            InteractionChannel::GibbsExcess,
            cross_check,
        )?,
        magnetic: load_phase_interactions_with_cross_check(
            engine,
            phase_index,
            InteractionChannel::Magnetic,
            cross_check,
        )?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    enum SyntheticCrossCheckOutcome {
        Descriptor(InteractionDescriptor),
        Unavailable,
        Error(String),
    }

    struct SyntheticDatCrossCheck {
        outcome: SyntheticCrossCheckOutcome,
    }

    impl InteractionDescriptorCrossCheck for SyntheticDatCrossCheck {
        fn descriptor_for(
            &self,
            _request: InteractionCrossCheckRequest<'_>,
        ) -> Result<Option<InteractionDescriptor>, String> {
            match &self.outcome {
                SyntheticCrossCheckOutcome::Descriptor(descriptor) => Ok(Some(descriptor.clone())),
                SyntheticCrossCheckOutcome::Unavailable => Ok(None),
                SyntheticCrossCheckOutcome::Error(reason) => Err(reason.clone()),
            }
        }
    }

    fn powered_descriptor(order: InteractionOrder) -> InteractionDescriptor {
        InteractionDescriptor::Powered {
            parameter_index: 1,
            declared_arity: 2,
            first_sublattice: vec![
                NativePoweredMember {
                    index: NativeInteractionIndex(1),
                    order,
                },
                NativePoweredMember {
                    index: NativeInteractionIndex(2),
                    order: InteractionOrder::Numeric(0),
                },
            ],
            following_sublattices: Vec::new(),
            type_label: None,
        }
    }

    fn synthetic_raw(native_descriptor: &str, values: Vec<Vec<f64>>) -> InteractionRaw {
        InteractionRaw {
            phase_index: 1,
            phase_name: "Synthetic".to_owned(),
            model: "QKTO".to_owned(),
            sublattice_count: 1,
            channel: InteractionChannel::GibbsExcess,
            parameter_index: 1,
            raw_descriptor: native_descriptor.to_owned(),
            values,
        }
    }

    fn metadata() -> InteractionMetadata {
        InteractionMetadata {
            phase_constituents: vec!["PC1".into(), "PC2".into(), "PC3".into()],
            sublattices: vec![
                vec!["A".into(), "B".into(), "C".into()],
                vec!["D".into(), "E".into()],
                vec!["F".into()],
            ],
        }
    }

    fn phase_constituent_metadata() -> InteractionMetadata {
        InteractionMetadata {
            phase_constituents: vec!["PC1".into(), "PC2".into(), "PC3".into()],
            // Phase-constituent interaction models such as QKTO/QKTOM have
            // one interaction-bearing sublattice even though resolution uses
            // the phase-constituent namespace.
            sublattices: vec![vec!["unused".into()]],
        }
    }

    #[test]
    fn parses_powered_members_sublattice_groups_and_unknown_label() {
        let descriptor =
            parse_interaction_descriptor("17: *2 (2)^[*]-(3)^[0] : (5) (Future-Label)");
        match descriptor {
            InteractionDescriptor::Powered {
                parameter_index,
                first_sublattice,
                following_sublattices,
                type_label,
                ..
            } => {
                assert_eq!(parameter_index, 17);
                assert_eq!(first_sublattice[0].order, InteractionOrder::Wildcard);
                assert_eq!(following_sublattices, vec![vec![NativeInteractionIndex(5)]]);
                assert_eq!(type_label.as_deref(), Some("Future-Label"));
            }
            other => panic!("unexpected descriptor: {other:?}"),
        }
    }

    #[test]
    fn transformed_descriptor_omits_native_index_arity_and_identity_annotations() {
        let metadata = InteractionMetadata {
            phase_constituents: Vec::new(),
            sublattices: vec![
                vec!["Al".into(), "Si".into(), "Ca".into()],
                vec!["O".into()],
            ],
        };
        let parsed = parse_interaction_descriptor("1: *2 (1)^[0]-(3)^[0] : (4) (Guts)");
        let resolved =
            resolve_descriptor(&parsed, ResolutionStrategy::FlattenedSublattices, &metadata)
                .unwrap();

        assert_eq!(resolved.to_string(), "(Al)^[0]-(Ca)^[0] : (O) (Guts)");
    }

    #[test]
    fn transformed_unpowered_sublattice_groups_use_the_same_colon_structure() {
        let parsed = parse_interaction_descriptor("1: *2 (1)-(2) : (4)");
        let metadata = InteractionMetadata {
            phase_constituents: Vec::new(),
            sublattices: vec![vec!["A".into(), "B".into(), "C".into()], vec!["D".into()]],
        };
        let resolved =
            resolve_descriptor(&parsed, ResolutionStrategy::FlattenedSublattices, &metadata)
                .unwrap();

        assert_eq!(resolved.to_string(), "(A)-(B) : (D)");
        assert_eq!(parsed.sublattice_count(), Some(2));
        assert_eq!(resolved.sublattice_count(), 2);
    }

    #[test]
    fn transformed_olivine_style_descriptor_preserves_four_sublattice_groups() {
        let metadata = InteractionMetadata {
            phase_constituents: Vec::new(),
            sublattices: vec![
                vec!["Ca".into(), "Fe".into()],
                vec!["Ca".into()],
                vec!["Si".into()],
                vec!["O".into()],
            ],
        };
        let parsed = parse_interaction_descriptor("1: *2 (1)-(2) : (3) : (4) : (5)");
        let resolved =
            resolve_descriptor(&parsed, ResolutionStrategy::FlattenedSublattices, &metadata)
                .unwrap();

        assert_eq!(resolved.to_string(), "(Ca)-(Fe) : (Ca) : (Si) : (O)");
        assert_eq!(parsed.sublattice_count(), Some(4));
        assert_eq!(resolved.sublattice_count(), 4);
    }

    #[test]
    fn transformed_single_group_has_no_colon() {
        let parsed = parse_interaction_descriptor("1: *2 (1)^[0]-(2)^[1]");
        let resolved = resolve_descriptor(
            &parsed,
            ResolutionStrategy::PhaseConstituents,
            &phase_constituent_metadata(),
        )
        .unwrap();

        assert_eq!(resolved.to_string(), "(PC1)^[0]-(PC2)^[1]");
        assert!(!resolved.to_string().contains(':'));
        assert_eq!(parsed.sublattice_count(), Some(1));
        assert_eq!(resolved.sublattice_count(), 1);
    }

    #[test]
    fn two_digit_order_difference_is_a_typed_validated_recovery() {
        for power in [10usize, 15usize] {
            let native_descriptor = "1: *2 (1)^[*]-(2)^[0]";
            let values = vec![vec![1234.5, -6.0]];
            let native_parsed = powered_descriptor(InteractionOrder::Wildcard);
            let dat_descriptor = powered_descriptor(InteractionOrder::Numeric(power));
            let provider = SyntheticDatCrossCheck {
                outcome: SyntheticCrossCheckOutcome::Descriptor(dat_descriptor.clone()),
            };
            let interaction = interpret_interaction(
                synthetic_raw(native_descriptor, values.clone()),
                native_parsed.clone(),
                Some(&provider),
                ResolutionStrategy::PhaseConstituents,
                &phase_constituent_metadata(),
            );

            assert_eq!(interaction.raw.raw_descriptor, native_descriptor);
            assert_eq!(interaction.raw.values, values);
            assert_eq!(interaction.native_parsed, native_parsed);
            assert_eq!(interaction.effective_descriptor, dat_descriptor);
            assert_eq!(
                interaction.effective_source(),
                InteractionDescriptorSource::DatRecovered
            );
            assert!(matches!(
                interaction.cross_check,
                InteractionCrossCheck::ValidatedRecovery {
                    reason: InteractionRecoveryReason::KnownMultiDigitOrderCorruption,
                    ..
                }
            ));
            assert!(interaction.resolved_text().contains("PC1"));
            assert!(interaction.resolved_text().contains(&format!("^[{power}]")));
        }
    }

    #[test]
    fn wildcard_agreement_is_not_recovery() {
        let native = powered_descriptor(InteractionOrder::Wildcard);
        let provider = SyntheticDatCrossCheck {
            outcome: SyntheticCrossCheckOutcome::Descriptor(native.clone()),
        };
        let interaction = interpret_interaction(
            synthetic_raw("1: *2 (1)^[*]-(2)^[0]", vec![vec![1.0]]),
            native.clone(),
            Some(&provider),
            ResolutionStrategy::PhaseConstituents,
            &phase_constituent_metadata(),
        );

        assert_eq!(interaction.cross_check, InteractionCrossCheck::Agree);
        assert_eq!(interaction.effective_descriptor, native);
        assert_eq!(
            interaction.effective_source(),
            InteractionDescriptorSource::Native
        );
    }

    #[test]
    fn ordinary_power_disagreements_keep_native_effective() {
        for (native_order, dat_order) in [
            (InteractionOrder::Wildcard, InteractionOrder::Numeric(3)),
            (InteractionOrder::Numeric(2), InteractionOrder::Numeric(3)),
        ] {
            let native = powered_descriptor(native_order);
            let dat = powered_descriptor(dat_order);
            let provider = SyntheticDatCrossCheck {
                outcome: SyntheticCrossCheckOutcome::Descriptor(dat.clone()),
            };
            let interaction = interpret_interaction(
                synthetic_raw("synthetic native text", vec![vec![7.0]]),
                native.clone(),
                Some(&provider),
                ResolutionStrategy::PhaseConstituents,
                &phase_constituent_metadata(),
            );

            assert!(matches!(
                interaction.cross_check,
                InteractionCrossCheck::Disagree {
                    ref dat_descriptor,
                    ..
                } if dat_descriptor == &dat
            ));
            assert_eq!(interaction.native_parsed, native);
            assert_eq!(interaction.effective_descriptor, native);
            assert_eq!(interaction.raw.values, vec![vec![7.0]]);
            assert!(interaction.is_resolved());
        }
    }

    #[test]
    fn provider_error_and_unavailable_data_leave_healthy_native_resolved() {
        let native = powered_descriptor(InteractionOrder::Numeric(2));
        let make_interaction = |outcome| {
            let provider = SyntheticDatCrossCheck { outcome };
            interpret_interaction(
                synthetic_raw("1: *2 (1)^[2]-(2)^[0]", vec![vec![1234.5]]),
                native.clone(),
                Some(&provider),
                ResolutionStrategy::PhaseConstituents,
                &phase_constituent_metadata(),
            )
        };
        let failed = make_interaction(SyntheticCrossCheckOutcome::Error(
            "synthetic lookup failure".to_owned(),
        ));
        let unavailable = make_interaction(SyntheticCrossCheckOutcome::Unavailable);

        assert!(matches!(
            failed.cross_check,
            InteractionCrossCheck::DatError { ref reason }
                if reason == "synthetic lookup failure"
        ));
        assert_eq!(unavailable.cross_check, InteractionCrossCheck::Unavailable);
        for interaction in [failed, unavailable] {
            assert_eq!(interaction.native_parsed, native);
            assert_eq!(interaction.effective_descriptor, native);
            assert_eq!(interaction.raw.values, vec![vec![1234.5]]);
            assert!(interaction.is_resolved());
        }
    }

    #[test]
    fn unparsed_native_descriptor_is_not_automatically_recovered() {
        let native_text = "(Si)";
        let native = parse_interaction_descriptor(native_text);
        let dat = powered_descriptor(InteractionOrder::Numeric(10));
        let provider = SyntheticDatCrossCheck {
            outcome: SyntheticCrossCheckOutcome::Descriptor(dat.clone()),
        };
        let values = vec![vec![42.0, -1.0]];
        let interaction = interpret_interaction(
            synthetic_raw(native_text, values.clone()),
            native.clone(),
            Some(&provider),
            ResolutionStrategy::PhaseConstituents,
            &phase_constituent_metadata(),
        );

        assert!(matches!(native, InteractionDescriptor::Unparsed { .. }));
        assert_eq!(interaction.native_parsed, native);
        assert_eq!(interaction.effective_descriptor, native);
        assert_eq!(interaction.raw.raw_descriptor, native_text);
        assert_eq!(interaction.raw.values, values);
        assert!(matches!(
            interaction.cross_check,
            InteractionCrossCheck::Disagree { ref dat_descriptor, .. }
                if dat_descriptor == &dat
        ));
        assert!(!interaction.is_resolved());
    }

    #[test]
    fn parses_reciprocal_sides_without_flattening_them() {
        let descriptor = parse_interaction_descriptor("4: *R (1)-(3) : (6)-(8) (Reciprocal)");
        assert!(matches!(
            descriptor,
            InteractionDescriptor::Reciprocal {
                left: [NativeInteractionIndex(1), NativeInteractionIndex(3)],
                right: [NativeInteractionIndex(6), NativeInteractionIndex(8)],
                ..
            }
        ));
    }

    #[test]
    fn parses_three_sublattice_groups_and_rejects_trailing_syntax() {
        let descriptor = parse_interaction_descriptor("1: *2 (1)-(2) : (4) : (6)");
        assert!(matches!(
            descriptor,
            InteractionDescriptor::SublatticeGroups { ref sublattices, .. }
                if sublattices.len() == 3
        ));
        assert!(matches!(
            parse_interaction_descriptor("1: *2 (1)^[0]-(2)^[0] trailing"),
            InteractionDescriptor::Unparsed { .. }
        ));
    }

    #[test]
    fn flattened_resolver_handles_every_boundary_and_three_sublattices() {
        let metadata = metadata();
        let expected = [(1, 1, 1), (3, 1, 3), (4, 2, 1), (5, 2, 2), (6, 3, 1)];
        for (encoded, sublattice, local_index) in expected {
            assert!(matches!(
                resolve_flattened(NativeInteractionIndex(encoded), &metadata.sublattices).unwrap(),
                InteractionMember::SublatticeSpecies { sublattice: actual_sublattice, local_index: actual_local, .. }
                    if actual_sublattice == sublattice && actual_local == local_index
            ));
        }
        assert!(resolve_flattened(NativeInteractionIndex(0), &metadata.sublattices).is_err());
        assert!(resolve_flattened(NativeInteractionIndex(7), &metadata.sublattices).is_err());
    }

    #[test]
    fn model_dispatch_keeps_phase_and_sublattice_namespaces_distinct() {
        assert_eq!(
            resolution_strategy("QKTO"),
            ResolutionStrategy::PhaseConstituents
        );
        assert_eq!(
            resolution_strategy("QKTOM"),
            ResolutionStrategy::PhaseConstituents
        );
        assert_eq!(
            resolution_strategy("SUBQ"),
            ResolutionStrategy::FlattenedSublattices
        );
        assert_eq!(
            resolution_strategy("SUBLM"),
            ResolutionStrategy::FlattenedSublattices
        );
        assert_eq!(resolution_strategy("IDMX"), ResolutionStrategy::Unsupported);

        let metadata = metadata();
        assert!(matches!(
            resolve_member(NativeInteractionIndex(2), ResolutionStrategy::PhaseConstituents, &metadata).unwrap(),
            InteractionMember::PhaseConstituent { name, .. } if name == "PC2"
        ));
        assert!(resolve_member(
            NativeInteractionIndex(0),
            ResolutionStrategy::PhaseConstituents,
            &metadata
        )
        .is_err());
        assert!(matches!(
            resolve_member(NativeInteractionIndex(2), ResolutionStrategy::FlattenedSublattices, &metadata).unwrap(),
            InteractionMember::SublatticeSpecies { name, .. } if name == "B"
        ));
    }

    #[test]
    fn typed_addresses_lower_to_the_official_tqcdat_selectors() {
        assert_eq!(
            InteractionParameterAddress::Gibbs {
                phase_index: 7,
                interaction_index: 4,
                expression_index: 2,
                term_index: 5,
            }
            .tqcdat_selectors()
            .unwrap(),
            [13, 4, 2, 5, 7]
        );
        assert_eq!(
            InteractionParameterAddress::Magnetic {
                phase_index: 3,
                interaction_index: 8,
                expression_index: 2,
                role: MagneticInteractionRole::CurieNeelTemperature,
            }
            .tqcdat_selectors()
            .unwrap(),
            [10, 8, 2, 1, 3]
        );
        assert_eq!(
            InteractionParameterAddress::Magnetic {
                phase_index: 3,
                interaction_index: 8,
                expression_index: 2,
                role: MagneticInteractionRole::MagneticMoment,
            }
            .tqcdat_selectors()
            .unwrap(),
            [10, 8, 2, 2, 3]
        );
    }

    #[test]
    fn typed_addresses_reject_zero_native_indices() {
        let addresses = [
            InteractionParameterAddress::Gibbs {
                phase_index: 0,
                interaction_index: 1,
                expression_index: 1,
                term_index: 1,
            },
            InteractionParameterAddress::Gibbs {
                phase_index: 1,
                interaction_index: 0,
                expression_index: 1,
                term_index: 1,
            },
            InteractionParameterAddress::Magnetic {
                phase_index: 1,
                interaction_index: 1,
                expression_index: 0,
                role: MagneticInteractionRole::MagneticMoment,
            },
        ];
        for address in addresses {
            assert!(address.tqcdat_selectors().is_err());
        }
    }

    #[test]
    fn parameter_cells_retain_full_matrices_and_mark_special_columns_read_only() {
        let mut raw = synthetic_raw("native", vec![vec![1.0; 18]]);
        raw.model = "SUBQ".to_owned();
        let interaction = Interaction {
            raw,
            native_parsed: powered_descriptor(InteractionOrder::Numeric(0)),
            cross_check: InteractionCrossCheck::NotRequested,
            effective_descriptor: powered_descriptor(InteractionOrder::Numeric(0)),
            resolution: InteractionResolution::Unresolved {
                reason: "not relevant".to_owned(),
            },
        };
        let cells = interaction.parameter_cells();
        assert_eq!(cells.len(), 18);
        assert!(matches!(
            cells[5].mutation,
            InteractionMutationSupport::Verified(InteractionParameterAddress::Gibbs {
                term_index: 6,
                ..
            })
        ));
        assert!(matches!(
            cells[6].mutation,
            InteractionMutationSupport::ReadOnly { .. }
        ));
        assert_eq!(cells[17].value, 1.0);
    }

    #[test]
    fn magnetic_cells_have_semantic_roles_not_generic_terms() {
        let mut raw = synthetic_raw("native", vec![vec![11.0, 22.0]]);
        raw.model = "SUBLM".to_owned();
        raw.channel = InteractionChannel::Magnetic;
        let interaction = Interaction {
            raw,
            native_parsed: powered_descriptor(InteractionOrder::Numeric(0)),
            cross_check: InteractionCrossCheck::NotRequested,
            effective_descriptor: powered_descriptor(InteractionOrder::Numeric(0)),
            resolution: InteractionResolution::Unresolved {
                reason: "not relevant".to_owned(),
            },
        };
        let cells = interaction.parameter_cells();
        assert_eq!(
            cells[0].role,
            InteractionParameterRole::CurieNeelTemperature
        );
        assert_eq!(cells[1].role, InteractionParameterRole::MagneticMoment);
    }
}
