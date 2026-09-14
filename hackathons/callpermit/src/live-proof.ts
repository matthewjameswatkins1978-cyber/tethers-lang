import type { JsonObject } from "@call-e/calle";

/**
 * CALL-E's publicly announced hackathon integration-test recipient.
 * This is a CALL-E-owned test route, not Matthew's number and not evidence
 * that the UK route is currently available.
 */
export const CALLE_OFFICIAL_TEST_HOTLINE = Object.freeze({
  profile: "official-hotline",
  phone: "+12763229632",
  region: "US",
  locale: "en-US",
  target_type: "official_hackathon_test_hotline",
});

/** Result fields requested for the harmless hotline integration proof. */
export const LIVE_PROOF_RESULT_SCHEMA: JsonObject = {
  type: "object",
  additionalProperties: false,
  properties: {
    connected: { type: "boolean" },
    test_response_observed: { type: "boolean" },
    notes: { type: "string" },
  },
  required: ["connected", "test_response_observed"],
};

export const LIVE_PROOF_PROMPT =
  "Call CALL-E's official hackathon integration-test hotline. State that this is a CALL-E integration test and obtain a brief test response. Do not make appointments, purchases, subscriptions, payments, deposits, or any other financial or external commitment. Return only the observed integration-test outcome as structured evidence.";

/** Refuse every live target except the explicit, published CALL-E profile. */
export function requireOfficialHotlineProfile(profile: string | undefined) {
  if (profile !== CALLE_OFFICIAL_TEST_HOTLINE.profile) {
    throw new Error(
      `CALLPERMIT_LIVE_PROFILE must be exactly ${CALLE_OFFICIAL_TEST_HOTLINE.profile}; refuse arbitrary live destinations`,
    );
  }
  return CALLE_OFFICIAL_TEST_HOTLINE;
}
