import assert from "node:assert/strict";
import test from "node:test";
import {
  REAL_VCF_MESSAGE,
  localGetJob,
  localListContacts,
  localListJobs,
  localValidateRules,
  readFixture,
  resetLocalFixtures,
} from "./local-fixtures.mjs";

test("elige fixture por query, almacenamiento o resultado", () => {
  assert.equal(readFixture("?fixture=vacio", "resultado"), "vacio");
  assert.equal(readFixture("", "error"), "error");
  assert.equal(readFixture("?fixture=nope", null), "resultado");
});

test("vacío, error y resultado son sintéticos", () => {
  resetLocalFixtures();
  assert.deepEqual(localListJobs("vacio"), { items: [] });
  assert.throws(() => localListJobs("error"), /Fixture sintético/);
  const listed = localListJobs("resultado");
  assert.equal(listed.items.length, 1);
  assert.equal(listed.items[0].job_id, "job-sintetico");
  assert.equal(listed.items[0].display_name, "agenda-sintetica.vcf");
  const job = localGetJob("resultado", "job-sintetico");
  assert.equal(job.job.status, "completed");
  const contacts = localListContacts("resultado", "job-sintetico");
  assert.equal(contacts.items.length, 2);
  assert.match(contacts.items[0].emails[0], /@example\.invalid$/);
});

test("un VCF real no se procesa en el adaptador local", () => {
  assert.match(REAL_VCF_MESSAGE, /zedazo-api/);
  assert.equal(localValidateRules("   ").ok, false);
  assert.equal(localValidateRules("[zedazo]\n").ok, true);
});
