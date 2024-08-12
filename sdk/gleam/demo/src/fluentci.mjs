import { dag } from "jsr:@fluentci/sdk";

export function client() {
  return dag;
}

export function setSecret(c, name, value) {
  return c.setSecret(name, value);
}

export function plaintext(secret) {
  return secret.plaintext();
}
