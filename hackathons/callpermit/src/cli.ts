import {
  buildAppointmentEnvelope,
} from "./authority.js";
import { buildMatchedTethersResponse, runAppointmentCall } from "./callpermit.js";
import { createCallRegistry } from "./calle.js";
import { createFakeCalleClient } from "./fake-calle.js";
import type { AppointmentTerms, CallResult, CalleCredentials } from "./types.js";

const terms: AppointmentTerms = {
  offered_date: "2026-09-15",
  offered_time: "15:30",
  offered_price_minor: 4500,
  currency: "GBP",
  service: "standard service",
  extra_requested: null,
};

const fakeResult: CallResult = {
  call_id: "fake-cli-call-1",
  status: "completed",
  outcome: "COMMITTED",
  commitment_made: true,
  task_completed: true,
  offered_date: terms.offered_date,
  offered_time: terms.offered_time,
  offered_price_minor: terms.offered_price_minor,
  currency: terms.currency,
  service: terms.service,
  extra_requested: terms.extra_requested,
  recipient_words_supporting_result: ["Standard service confirmed for Tuesday at 15:30 for £45."],
};

const envelope = buildAppointmentEnvelope();
const credentials: CalleCredentials = {
  api_key: "offline-placeholder",
  phone_number: "+15555550100",
};

const result = await runAppointmentCall({
  envelope,
  terms,
  tethers_response: buildMatchedTethersResponse(envelope, terms),
  credentials,
  registry: createCallRegistry(),
  destination: "+15555550101",
  client: createFakeCalleClient({ fixedResult: fakeResult }),
});

console.log(JSON.stringify({
  mode: "offline-fake-calle",
  live_call: false,
  tethers: result.authority,
  call: result.call_result,
  reconciliation: result.reconciliation,
}, null, 2));
