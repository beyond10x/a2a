---
title: UNMAPPED markers and ESS gaps
sidebar_label: UNMAPPED markers
description: How the specification records each rule of A2A it does not hold, and the two kinds of marker.
---

The specification does not leave out a rule of A2A silently. Where it cannot hold one, a YAML
comment beside the place the rule belongs starts with `UNMAPPED`, quotes the A2A source and says
why.

## Two kinds

| Marker | Means | Resolved by |
|---|---|---|
| `UNMAPPED (ESS gap):` | ESS has no construct for the rule. The comment names the construct and, where ESS refused a try, the refusal it gave. | An ESS release that has the construct. |
| `UNMAPPED:` | A2A itself leaves the answer open. | An upstream source that settles it. A marker is never resolved by guessing. |

The [marker index](../reference/unmapped.md) lists every marker in `ess/` with its line, its kind
and its first sentence, and links each one to the comment on GitHub.

## HTTP+JSON routes

A route of A2A's HTTP+JSON binding that ESS cannot serve at A2A's path is not a YAML comment but a
row of `ess/http-unmapped.tsv`, with its reason. `task check` fails on a difference between the
binding and A2A's paths that the file does not name, and on a named one that no longer differs.
[HTTP+JSON routes](../reference/http-routes.md) lists them.
