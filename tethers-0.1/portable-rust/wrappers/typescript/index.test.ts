import { strict as assert } from "node:assert";
import { parseResponse } from "./index.ts";
assert.equal(parseResponse('{"schema_version":"2","decision":"ALLOW"}').decision, "DENY");
assert.equal(parseResponse('{"schema_version":"1","decision":"ASK"}').decision, "ASK");
