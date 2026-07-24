let firebasePromise;
let messagingPromise;
let foregroundHandlerReady = false;

function serviceWorkerUrl() {
    const manifest = document.querySelector('link[rel="manifest"]');
    const base = manifest?.href || document.baseURI;
    return new URL("firebase-messaging-sw.js", base).toString();
}

async function messagingFor(config) {
    const [appSdk, messagingSdk] = await firebaseModules();
    if (!(await messagingSdk.isSupported())) {
        throw new Error("Push notifications are not supported by this browser.");
    }
    if (!messagingPromise) {
        messagingPromise = (async () => {
            const app = appSdk.getApps().length ? appSdk.getApp() : appSdk.initializeApp({
                apiKey: config.api_key,
                authDomain: config.auth_domain,
                projectId: config.project_id,
                storageBucket: config.storage_bucket,
                messagingSenderId: config.messaging_sender_id,
                appId: config.app_id,
            });
            const messaging = messagingSdk.getMessaging(app);
            if (!foregroundHandlerReady) {
                foregroundHandlerReady = true;
                messagingSdk.onMessage(messaging, (payload) => {
                    if (Notification.permission !== "granted" || !payload.notification) return;
                    const notification = new Notification(
                        payload.notification.title || "VL Rental update",
                        {
                            body: payload.notification.body || "Your booking has an update.",
                            icon: new URL("icon-192.png", document.baseURI).toString(),
                            tag: payload.data?.notification_type || "vl-rental-update",
                        },
                    );
                    notification.onclick = () => {
                        window.focus();
                        window.location.assign(payload.data?.url || new URL("account", document.baseURI));
                    };
                });
            }
            return { messaging, messagingSdk };
        })();
    }
    return messagingPromise;
}

function firebaseModules() {
    if (!firebasePromise) {
        firebasePromise = Promise.all([
            import("https://www.gstatic.com/firebasejs/12.16.0/firebase-app.js"),
            import("https://www.gstatic.com/firebasejs/12.16.0/firebase-messaging.js"),
        ]);
    }
    return firebasePromise;
}

async function tokenFor(config, requestPermission) {
    if (requestPermission && Notification.permission === "default") {
        await Notification.requestPermission();
    }
    if (Notification.permission !== "granted") return "";
    const registration = await navigator.serviceWorker.register(serviceWorkerUrl());
    const { messaging, messagingSdk } = await messagingFor(config);
    return await messagingSdk.getToken(messaging, {
        vapidKey: config.vapid_public_key,
        serviceWorkerRegistration: registration,
    });
}

window.vlPush = {
    permission() {
        if (!("Notification" in window) || !("serviceWorker" in navigator)) return "unsupported";
        return Notification.permission;
    },
    async subscribe(config) {
        return await tokenFor(config, true);
    },
    async currentToken(config) {
        return await tokenFor(config, false);
    },
    async unsubscribe(config) {
        if (Notification.permission !== "granted") return false;
        const { messaging, messagingSdk } = await messagingFor(config);
        return await messagingSdk.deleteToken(messaging);
    },
};

document.documentElement.dataset.vlPushClient = "ready";
