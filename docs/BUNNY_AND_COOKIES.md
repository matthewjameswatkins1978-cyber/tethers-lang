# Bunny & Cookies: the storybook tour of Tethers

Imagine a bunny has learned that pressing a button means “cookie please.” 🐇🍪

This example follows one request from the physical world into Tethers and back
again. It separates four things that are easy to accidentally blur together:

1. what an event source observed;
2. what the host admitted as a valid event;
3. what Tethers decided to request; and
4. what the provider and the physical world later reported.

## The whole story

```text
bunny presses button
        |
        v
event source notices it
        |
        v
event proposal
        |
        v
Tethers host admits it
        |
        v
anchor + facts
        |
        v
Tether evaluates conditions
        |
        v
action: cookie.dispense
        |
        v
cookie dispenser Plug calls provider
        |
        v
provider reports capability.succeeded
        |
        v
sensor observes a cookie falling
        |
        v
new event: cookie.dispensed
```

The last event can start another Tether. The machine has a beginning, a middle,
an end, a receipt, and a possible sequel.

## Chapter 1: before Tethers

The bunny first touches a physical button. A hardware driver or application
observes “button 1 went down”. An installed Event Source translates that
observation into a proposal:

```json
{
  "event_name": "bunny.button_pressed",
  "event_version": 1,
  "source_event_id": "button-48291",
  "facts": {
    "button": "cookie",
    "bunny": "Banana"
  }
}
```

The Event Source is making a claim: “I saw Banana press the cookie button.” It
does not decide what the press means, and Tethers does not pretend that an
observation is automatically the whole truth about the universe.

## Chapter 2: the front door

The proposal reaches the Tethers Host. Before it reaches a Tether, the host
checks the admission ticket:

- Is this an installed Event Source?
- Did this source declare that it can emit `bunny.button_pressed`?
- Do the facts match the declared schema?
- Is the payload within its size limits?
- Was the source launched with its authorised configuration?

If it passes, the host adds its own evidence: an event ID, admission time,
admission sequence, source identity, correlation information, generation, and a
Trail receipt. The result is now an admitted Tethers event, for example:

```text
event_name:          bunny.button_pressed
facts:               button = "cookie", bunny = "Banana"
event_id:            evt-91842
admission_sequence:  4821
```

Only now do we reach the part Tethers itself cares about.

## Chapter 3: the Anchor

Our Tether might be written conceptually as:

```text
tether "Give bunny a cookie"
anchor
    bunny.button_pressed
when
    button is "cookie"
do
    cookie.dispense
        amount: 1
```

The Anchor means “this Tether is interested in admitted events named
`bunny.button_pressed`.” It does not mean “watch the button”. Watching is the
Event Source’s job.

```text
Event Source = watches
Anchor       = listens
```

That distinction keeps the boundary clean.

## Chapter 4: inside Tethers

Tethers receives an admitted event with immutable facts:

```text
bunny  = Banana
button = cookie
```

It evaluates the condition `button is "cookie"`. The condition is true, so the
deterministic result is a Plan containing an action request:

```text
ACTION
  cookie.dispense
    amount: 1
```

Tethers has still not touched the dispenser. It has only decided that, given
this event and these facts, the next requested action is one cookie.

## Chapter 5: leaving Tethers

The action crosses an explicit output boundary:

```text
Tethers -> Action -> Capability -> Plug -> Provider -> physical world
```

The capability might be `cookie.dispense@1`. The host resolves an installed Plug
that provides it and checks the capability, enablement, policy, scope, and
provider identity. Only then may it invoke something such as
`cookie-provider.exe`:

```text
USB -> motor turns -> cookie
```

The Plug is the translator. Tethers does not need to know whether the provider
uses USB, a local process, a network call, or a completely different mechanism.

## Chapter 6: the action result

The provider may return:

```json
{
  "status": "dispensed",
  "amount": 1
}
```

The host validates that result and creates a `capability.succeeded` event whose
facts describe what happened when Tethers asked:

```text
capability.name = cookie.dispense
provider        = cookie-provider
result.amount   = 1
result.status   = dispensed
```

This is an Action Result: “the invocation completed successfully.” It is a
receipt about the request and provider response.

## Chapter 7: did a cookie really fall?

The dispenser can report that its motor operation succeeded while the hopper
was empty. A physical optical sensor provides a different kind of evidence. If
it actually sees a cookie pass, it emits another proposal:

```json
{
  "event_name": "cookie.dispensed",
  "event_version": 1,
  "facts": {
    "dispenser": "hutch-1",
    "count": 1
  }
}
```

That proposal comes through the normal event-source admission path. Another
Tether can react:

```text
tether "Record successful bunny treat"
anchor
    cookie.dispensed
when
    count is 1
do
    bunny.treat_record
        amount: 1
```

## The important distinction

There are two ways the outside world can appear to “come back”:

| Evidence | Meaning |
| --- | --- |
| `capability.succeeded` | What happened when Tethers asked a provider to do something. |
| `cookie.dispensed` | What a sensor or other Event Source says happened in reality. |

They may agree, and usually should. Tethers does not collapse them into one
claim. “I requested it” is not the same as “reality definitely changed”.

That separation is the deeper lesson: decisions, attempts/results, and later
observations remain distinct, inspectable pieces of evidence. The bunny can have
her cookie without the system having to lie about causality. 😂🐇🍪
