import { dag } from "@fluentci/sdk";

export { dag };

export function awsSecretsManager(c) {
  return function (client) {
    return function (region) {
      return function (accessKeyId) {
        return function (clientSecret) {
          return function (tenantId) {
            return function (keyvaultName) {
              return function (keyvaultUrl) {
                return function () {
                  return c.awsSecretsManager({
                    client,
                    region,
                    accessKeyId,
                    clientSecret,
                    tenantId,
                    keyvaultName,
                    keyvaultUrl,
                  });
                };
              };
            };
          };
        };
      };
    };
  };
}

export function azureKeyvault(c) {
  return function (clientId) {
    return function (clientSecret) {
      return function (tenantId) {
        return function (keyvaultName) {
          return function (keyvaultUrl) {
            return function () {
              return c.azureKeyvault({
                clientId,
                clientSecret,
                tenantId,
                keyvaultName,
                keyvaultUrl,
              });
            };
          };
        };
      };
    };
  };
}
export function cache(c) {
  return function (key) {
    return function () {
      return c.cache(key);
    };
  };
}

export function setSecret(c) {
  return function (name) {
    return function (value) {
      return function () {
        return c.setSecret(name, value);
      };
    };
  };
}
