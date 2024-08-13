export function _plaintext(s) {
  return function (onError, onSuccess) {
    s.plaintext().then(onSuccess).catch(onError);
    return function (cancelError, onCancelerError, onCancelerSuccess) {
      onCancelerSuccess();
    };
  };
}
