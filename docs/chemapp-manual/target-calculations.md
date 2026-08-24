# Temperature-target calculations

This page records the high-level numerical contract of
`Calculator::calculate_target_t`. The native target equilibrium remains a
ChemApp `TQCE("T", ...)` operation; Rust adds only an optional scalar incoming-
composition consistency solve around it.

## Native state and conditions

The helper does not call `TQREMC`. It inherits the active pressure, units,
phase and constituent statuses, and other existing ChemApp conditions. It sets
the requested phase-amount target (`A`), writes the requested temperature
limits, writes a complete set of incoming system-component amounts (`IA`), and
performs a temperature-target `TQCE`.

With no fixed/adjusting component pair, exactly one target equilibrium is
performed. With a pair, both component indices are distinct, positive,
one-based system-component indices. The incoming fixed-component amount and
all unrelated incoming amounts remain constant; only the adjusting-component
amount changes between trials. No composition renormalization is inserted.

## Self-consistency equation

For fixed component `f` and adjusting component `a`, define the incoming ratio

```text
q = IA_a / IA_f.
```

After a target equilibrium, let `X_f` and `X_a` be `TQGETR("XP", ...)` for the
two components in the selected master phase. The physical consistency equation
is

```text
q = X_a / X_f.
```

The former implementation repeatedly assigned the phase ratio to the incoming
ratio (with later damping). Such Picard iteration can converge, oscillate, or
diverge depending on the local response, and a small damped step does not prove
that the physical equation is satisfied. It also incorrectly returned success
after its fixed iteration count was exhausted.

For strictly positive ratios, the current solver uses

```text
y = ln(q)
R(y) = y - ln(X_a / X_f).
```

This residual is relative in scale: a factor-of-two error has the same meaning
for a major component and a trace component. Success requires
`abs(R) <= 1e-6`, approximately one part per million in the ratio. This is
tighter than the accuracy needed for ordinary thermochemical repeatability
without pretending that a native equilibrium is reproducible to f64 machine
epsilon.

## Safeguarded derivative-free solve

The original composition is evaluated first. Its phase ratio supplies the
first physically meaningful Picard predictor. Before a sign-changing residual
bracket exists, later useful points permit secant proposals. Every unbracketed
change is limited to `abs(delta y) <= ln(100)`, so a single trial cannot change
the adjusting amount by more than a factor of 100. A nearly singular secant or
a duplicate proposal falls back to bounded conservative exploration.

Once residuals of opposite sign are available, their narrowest known interval
is preserved. A secant point is accepted only inside a guarded interior of the
bracket; otherwise the next point is its midpoint. This bracketed secant/
bisection hybrid guarantees interval reduction without assuming a smooth
derivative. Each residual evaluation writes one complete IA composition,
performs exactly one target `TQCE`, and then reads the two XP values.

The private default budget is 32 target equilibria. Exhausting it returns a
contextual `ChemAppError` with the evaluation count, best residual and ratios,
and the master/fixed/adjusting indices. Budget exhaustion is never success.
Native errors from a trial propagate immediately; they are not reinterpreted as
requests for a smaller numerical step.

## Exact zero and invalid values

Zero is handled as a physical boundary rather than replaced with an epsilon:

- a non-positive or non-finite incoming fixed amount is an error;
- a non-positive or non-finite fixed-component master-phase fraction is an
  error because the phase ratio is undefined;
- zero incoming adjusting amount with zero adjusting phase fraction is an exact
  zero-ratio solution;
- zero incoming adjusting amount with positive adjusting phase fraction uses
  that phase ratio as the first positive seed;
- positive incoming adjusting amount with zero adjusting phase fraction causes
  an explicit trial of the zero boundary, never `ln(0)`.

Incoming amounts, phase fractions, ratios, logarithms, residuals, and proposed
ratios are checked for finite values. Unexpected NaN or infinity is a
contextual error.

## State after return

On success, the final converged native trial remains the live ChemApp state. On
failure there is no hidden rollback: the last successful or partial native
trial and previously written conditions may remain live. Callers that require
a known state after an error must explicitly establish it.
