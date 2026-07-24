importScripts("https://www.gstatic.com/firebasejs/12.16.0/firebase-app-compat.js");
importScripts("https://www.gstatic.com/firebasejs/12.16.0/firebase-messaging-compat.js");

firebase.initializeApp({
    apiKey: "AIzaSyB4li_UUArcknjgWwIAXz9Kwr9uGYGiyq4",
    authDomain: "vl-rental.firebaseapp.com",
    projectId: "vl-rental",
    storageBucket: "vl-rental.firebasestorage.app",
    messagingSenderId: "304296325622",
    appId: "1:304296325622:web:83fda2c73560b58331d7d7",
});

firebase.messaging();
