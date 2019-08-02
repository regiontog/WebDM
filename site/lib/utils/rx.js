// @flow

type IStream<T> = {
    (value?: T): ?T;
    map<R>(fn: (T) => R): IStream<R>;
    forEach(fn: (T) => void): void;
    inactive(duration: number): IStream<null>;
}

export function Stream<T>(value?: T): IStream<T> {
    let ref: ?T = value;

    const listeners: Array<(T) => void> = [];

    return Object.assign((value?: T) => {
        if (typeof (value) !== "undefined") {
            ref = value;
            listeners.forEach(listener => listener(value));
        } else {
            return ref;
        }
    }, {
            map<R>(fn: (T) => R): IStream<R> {
                const s = Stream();
                this.forEach(value => s(fn(value)));

                return s;
            },
            forEach(fn: (T) => void): void {
                listeners.push(fn);
            },
            inactive(duration: number): IStream<null> {
                const timeout = Stream();

                let timer = setInterval(() => timeout(null), duration);

                this.forEach(value => {
                    clearInterval(timer);
                    timer = setInterval(() => timeout(null), duration);
                });

                return timeout;
            }
        });
}

Stream.merge = function <T>(...observables: Array<IStream<T>>): IStream<T> {
    const s = Stream();
    observables.forEach(obs => obs.forEach(value => { s(value); }));

    return s;
}
