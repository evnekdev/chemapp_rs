//! Model-aware inspection of ChemApp excess Gibbs and magnetic interactions.
//!
//! The interaction path is deliberately additive: [`InteractionRaw`] retains
//! the exact TQLPAR text and authoritative live TQGPAR coefficients, parsing describes
//! the native indexed structure, and resolution maps those indices to names
//! obtained from ChemApp metadata. Unknown syntax is retained as unparsed data
//! instead of being discarded.

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

/// Context supplied to an optional ASCII-DAT descriptor recovery provider.
///
/// Providers should map the one-based TQLPAR/TQGPAR parameter index to a DAT
/// semantic interaction deterministically and validate that mapping on healthy
/// native descriptors before replacing a malformed descriptor.
#[derive(Clone, Copy, Debug)]
pub struct InteractionRecoveryRequest<'a> {
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

/// Optional boundary for recovering interaction structure from a compatible
/// ASCII DAT model.
///
/// The base crate remains usable with CST/BIN inputs and has no mandatory DAT
/// parser dependency. A provider returns the DAT descriptor for this exact
/// phase/channel/index, or `None` when unavailable. It must never supply
/// numerical parameters: live TQGPAR output remains authoritative. A provider
/// error marks only that row unresolved; it does not discard the row or abort
/// the surrounding inventory.
pub trait InteractionDescriptorRecovery {
    /// Return the deterministic DAT-side descriptor corresponding to this
    /// native interaction.
    fn recover_descriptor(
        &self,
        request: InteractionRecoveryRequest<'_>,
    ) -> Result<Option<InteractionDescriptor>, String>;
}

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

/// One complete raw → parsed → resolved interaction.
#[derive(Clone, Debug, PartialEq)]
pub struct Interaction {
    /// Authoritative native descriptor, identity, and coefficient matrix.
    pub raw: InteractionRaw,
    /// Additive structural interpretation of the native descriptor.
    pub parsed: InteractionDescriptor,
    /// Origin of the structural descriptor used for resolution.
    pub descriptor_source: InteractionDescriptorSource,
    /// Additive model-aware name-resolution outcome.
    pub resolution: InteractionResolution,
}

impl Interaction {
    /// True only when both parsing and model-aware name resolution succeeded.
    pub fn is_resolved(&self) -> bool {
        matches!(self.resolution, InteractionResolution::Resolved(_))
    }

    /// Render the deterministic name-based form or an explicit unresolved
    /// diagnostic; the raw descriptor is always available separately.
    pub fn resolved_text(&self) -> String {
        match &self.resolution {
            InteractionResolution::Resolved(value) => value.to_string(),
            InteractionResolution::Unresolved { reason } => format!("UNRESOLVED: {reason}"),
        }
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
                    interaction.parsed.kind_name().to_owned(),
                    if interaction.is_resolved() {
                        interaction.descriptor_source.to_string()
                    } else {
                        "Unresolved".to_owned()
                    },
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
                "Source",
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

fn descriptor_with_recovery(
    raw: &InteractionRaw,
    native_parsed: InteractionDescriptor,
    recovery: Option<&dyn InteractionDescriptorRecovery>,
) -> (
    InteractionDescriptor,
    InteractionDescriptorSource,
    Option<String>,
) {
    let Some(recovery) = recovery else {
        return (native_parsed, InteractionDescriptorSource::Native, None);
    };
    let recovered = match recovery.recover_descriptor(InteractionRecoveryRequest {
        phase_index: raw.phase_index,
        phase_name: &raw.phase_name,
        model: &raw.model,
        channel: raw.channel,
        parameter_index: raw.parameter_index,
        native_descriptor: &raw.raw_descriptor,
        native_parsed: &native_parsed,
    }) {
        Ok(recovered) => recovered,
        Err(reason) => {
            return (
                native_parsed,
                InteractionDescriptorSource::Native,
                Some(format!(
                    "interaction DAT recovery failed for phase {} ({}) {} parameter {}: {reason}",
                    raw.phase_name, raw.model, raw.channel, raw.parameter_index
                )),
            )
        }
    };
    let Some(recovered) = recovered else {
        return (native_parsed, InteractionDescriptorSource::Native, None);
    };
    if recovered.parameter_index() != Some(raw.parameter_index) {
        return (
            native_parsed,
            InteractionDescriptorSource::Native,
            Some(format!(
                "interaction DAT recovery returned the wrong parameter index for phase {} ({}) {} parameter {}",
                raw.phase_name, raw.model, raw.channel, raw.parameter_index
            )),
        );
    }
    if recovered == native_parsed {
        (native_parsed, InteractionDescriptorSource::Native, None)
    } else {
        (recovered, InteractionDescriptorSource::DatRecovered, None)
    }
}

pub(crate) fn load_phase_interactions(
    engine: &Engine,
    phase_index: usize,
    channel: InteractionChannel,
) -> Result<Vec<Interaction>, ChemAppError> {
    load_phase_interactions_with_recovery(engine, phase_index, channel, None)
}

pub(crate) fn load_phase_interactions_with_recovery(
    engine: &Engine,
    phase_index: usize,
    channel: InteractionChannel,
    recovery: Option<&dyn InteractionDescriptorRecovery>,
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
            let (parsed, descriptor_source, recovery_failure) =
                descriptor_with_recovery(&raw, parsed, recovery);
            let resolution = match recovery_failure {
                Some(reason) => InteractionResolution::Unresolved { reason },
                None => match resolve_descriptor(&parsed, strategy, &metadata) {
                    Ok(value) => InteractionResolution::Resolved(value),
                    Err(reason) => InteractionResolution::Unresolved { reason },
                },
            };
            Ok(Interaction {
                raw,
                parsed,
                descriptor_source,
                resolution,
            })
        })
        .collect()
}

pub(crate) fn load_phase_interaction_report_with_recovery(
    engine: &Engine,
    phase_index: usize,
    recovery: Option<&dyn InteractionDescriptorRecovery>,
) -> Result<PhaseInteractionReport, ChemAppError> {
    let phase_name = engine.tqgnp(phase_index)?;
    let model = engine.tqmodl(phase_index)?;
    let sublattice_count = engine.tqnosl(phase_index)?;
    Ok(PhaseInteractionReport {
        phase_index,
        phase_name,
        model,
        sublattice_count,
        gibbs: load_phase_interactions_with_recovery(
            engine,
            phase_index,
            InteractionChannel::GibbsExcess,
            recovery,
        )?,
        magnetic: load_phase_interactions_with_recovery(
            engine,
            phase_index,
            InteractionChannel::Magnetic,
            recovery,
        )?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SyntheticDatRecovery {
        power: usize,
    }

    struct FailedDatRecovery;

    impl InteractionDescriptorRecovery for SyntheticDatRecovery {
        fn recover_descriptor(
            &self,
            request: InteractionRecoveryRequest<'_>,
        ) -> Result<Option<InteractionDescriptor>, String> {
            Ok(Some(InteractionDescriptor::Powered {
                parameter_index: request.parameter_index,
                declared_arity: 2,
                first_sublattice: vec![
                    NativePoweredMember {
                        index: NativeInteractionIndex(1),
                        order: InteractionOrder::Numeric(self.power),
                    },
                    NativePoweredMember {
                        index: NativeInteractionIndex(2),
                        order: InteractionOrder::Numeric(0),
                    },
                ],
                following_sublattices: Vec::new(),
                type_label: None,
            }))
        }
    }

    impl InteractionDescriptorRecovery for FailedDatRecovery {
        fn recover_descriptor(
            &self,
            _request: InteractionRecoveryRequest<'_>,
        ) -> Result<Option<InteractionDescriptor>, String> {
            Err("synthetic lookup failure".to_owned())
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
    fn dat_recovery_retains_corrupt_native_text_and_live_values() {
        for (native_descriptor, power) in [("(Si)", 10usize), ("1: *2 (1)^[*]-(2)^[0]", 15usize)] {
            let values = vec![vec![1234.5, -6.0]];
            let raw = InteractionRaw {
                phase_index: 1,
                phase_name: "Synthetic".to_owned(),
                model: "QKTO".to_owned(),
                sublattice_count: 1,
                channel: InteractionChannel::GibbsExcess,
                parameter_index: 1,
                raw_descriptor: native_descriptor.to_owned(),
                values: values.clone(),
            };
            let native_parsed = parse_interaction_descriptor(native_descriptor);
            let recovery = SyntheticDatRecovery { power };
            let (parsed, source, recovery_failure) =
                descriptor_with_recovery(&raw, native_parsed, Some(&recovery));
            assert_eq!(recovery_failure, None);
            let resolution = resolve_descriptor(
                &parsed,
                ResolutionStrategy::PhaseConstituents,
                &phase_constituent_metadata(),
            )
            .unwrap();
            let interaction = Interaction {
                raw,
                parsed,
                descriptor_source: source,
                resolution: InteractionResolution::Resolved(resolution),
            };

            assert_eq!(interaction.raw.raw_descriptor, native_descriptor);
            assert_eq!(interaction.raw.values, values);
            assert_eq!(
                interaction.descriptor_source,
                InteractionDescriptorSource::DatRecovered
            );
            assert!(matches!(
                interaction.parsed,
                InteractionDescriptor::Powered { ref first_sublattice, .. }
                    if first_sublattice[0].order == InteractionOrder::Numeric(power)
            ));
            assert!(interaction.resolved_text().contains("PC1"));
            assert!(interaction.resolved_text().contains(&format!("^[{power}]")));
        }
    }

    #[test]
    fn dat_recovery_failure_preserves_native_row_for_unresolved_reporting() {
        let native_descriptor = "(Si)";
        let raw = InteractionRaw {
            phase_index: 1,
            phase_name: "Synthetic".to_owned(),
            model: "QKTO".to_owned(),
            sublattice_count: 1,
            channel: InteractionChannel::GibbsExcess,
            parameter_index: 1,
            raw_descriptor: native_descriptor.to_owned(),
            values: vec![vec![1234.5]],
        };
        let native_parsed = parse_interaction_descriptor(native_descriptor);
        let (parsed, source, recovery_failure) =
            descriptor_with_recovery(&raw, native_parsed.clone(), Some(&FailedDatRecovery));

        assert_eq!(parsed, native_parsed);
        assert_eq!(source, InteractionDescriptorSource::Native);
        assert!(recovery_failure
            .as_deref()
            .is_some_and(|reason| reason.contains("synthetic lookup failure")));
        assert_eq!(raw.raw_descriptor, native_descriptor);
        assert_eq!(raw.values, vec![vec![1234.5]]);
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
}
