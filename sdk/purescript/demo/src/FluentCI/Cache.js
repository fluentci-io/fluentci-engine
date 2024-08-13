export function _id(c) {
  return function (id) {
    return function (onError, onSuccess) {
      c.id(id).then(onSuccess).catch(onError);
      return function (cancelError, onCancelerError, onCancelerSuccess) {
        onCancelerSuccess();
      };
    };
  };
}

export function _key(c) {
  return function (onError, onSuccess) {
    c.key().then(onSuccess).catch(onError);
    return function (cancelError, onCancelerError, onCancelerSuccess) {
      onCancelerSuccess();
    };
  };
}
