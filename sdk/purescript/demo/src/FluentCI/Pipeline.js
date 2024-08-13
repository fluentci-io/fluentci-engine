export function asService(s) {
  return function () {
    return s.asService();
  };
}

export function _id(s) {
  return function (onError, onSuccess) {
    s.id().then(onSuccess).catch(onError);
    return function (cancelError, onCancelerError, onCancelerSuccess) {
      onCancelerSuccess();
    };
  };
}

export function devbox(s) {
  return function () {
    return s.devbox();
  };
}

export function devenv(s) {
  return function () {
    return s.devenv();
  };
}

export function envhub(s) {
  return function () {
    return s.envhub();
  };
}

export function mise(s) {
  return function () {
    return s.mise();
  };
}

export function nix(s) {
  return function () {
    return s.nix();
  };
}

export function pixi(s) {
  return function () {
    return s.pixi();
  };
}

export function pkgx(s) {
  return function () {
    return s.pkgx();
  };
}

export function _stderr(s) {
  return function (onError, onSuccess) {
    s.stderr().then(onSuccess).catch(onError);
    return function (cancelError, onCancelerError, onCancelerSuccess) {
      onCancelerSuccess();
    };
  };
}

export function _stdout(s) {
  return function (onError, onSuccess) {
    s.stdout().then(onSuccess).catch(onError);
    return function (cancelError, onCancelerError, onCancelerSuccess) {
      onCancelerSuccess();
    };
  };
}

export function waitOn(s) {
  return function (port) {
    return function (timeout) {
      return function () {
        return s.waitOn(port, timeout);
      };
    };
  };
}

export function withCache(s) {
  return function (cache) {
    return function () {
      return s.withCache(cache);
    };
  };
}

export function withEnvVariable(s) {
  return function (name) {
    return function (value) {
      return function () {
        return s.withEnvVariable(name, value);
      };
    };
  };
}

export function withSecretVariable(s) {
  return function (name) {
    return function (secret) {
      return function () {
        return s.withSecretVariable(name, secret);
      };
    };
  };
}

export function withExec(s) {
  return function (command) {
    return function () {
      return s.withExec(command);
    };
  };
}

export function withService(s) {
  return function (service) {
    return function () {
      return s.withService(service);
    };
  };
}

export function withWorkdir(s) {
  return function (path) {
    return function () {
      return s.withWorkdir(path);
    };
  };
}
