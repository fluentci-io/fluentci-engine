export function _id(s) {
  return function (onError, onSuccess) {
    s.id().then(onSuccess).catch(onError);
    return function (cancelError, onCancelerError, onCancelerSuccess) {
      onCancelerSuccess();
    };
  };
}

export function branch(s) {
  return function (name) {
    return function () {
      return s.branch();
    };
  };
}

export function _commit(s) {
  return function (onError, onSuccess) {
    s.commit().then(onSuccess).catch(onError);
    return function (cancelError, onCancelerError, onCancelerSuccess) {
      onCancelerSuccess();
    };
  };
}

export function tree(s) {
  return function () {
    return s.tree();
  };
}
