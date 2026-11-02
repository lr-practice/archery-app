// SafariWebExtensionHandler.swift
// App-Erweiterung der Safari-Extension in Archery.app. Safari schickt
// browser.runtime.sendNativeMessage nur hierher, nie an ein fremdes Programm.
// Die Erweiterung läuft in einer Sandbox und kann die Bibliothek nicht lesen;
// alles Weitere läuft per HTTP über die laufende App (http_bridge.rs).
// Hier gibt es genau einen Befehl:
//
//   { "cmd": "launch" }  Archery.app öffnen bzw. nach vorn holen.
//
// Die App wird gefunden, indem wir von der Erweiterung aus nach oben gehen:
// Archery.app/Contents/PlugIns/Archery Extension.appex → Archery.app.
// safari/build.sh ersetzt damit die Vorlage von safari-web-extension-converter.

import AppKit
import SafariServices
import os.log

class SafariWebExtensionHandler: NSObject, NSExtensionRequestHandling {

    func beginRequest(with context: NSExtensionContext) {
        let item = context.inputItems.first as? NSExtensionItem
        let message: Any?
        if #available(macOS 11.0, *) {
            message = item?.userInfo?[SFExtensionMessageKey]
        } else {
            message = item?.userInfo?["message"]
        }
        let cmd = (message as? [String: Any])?["cmd"] as? String
        guard cmd == "launch" else {
            reply(context, ["ok": false, "error": "Unbekannter Befehl"])
            return
        }
        guard let app = containingApp() else {
            os_log(.error, "Archery: Archery.app nicht gefunden")
            reply(context, ["ok": false, "error": "Archery.app nicht gefunden"])
            return
        }
        if #available(macOS 10.15, *) {
            let config = NSWorkspace.OpenConfiguration()
            config.activates = true
            NSWorkspace.shared.openApplication(at: app, configuration: config) { _, error in
                if let error = error {
                    os_log(.error, "Archery: Start fehlgeschlagen: %{public}@", error.localizedDescription)
                    self.reply(context, ["ok": false, "error": error.localizedDescription])
                } else {
                    self.reply(context, ["ok": true])
                }
            }
        } else {
            let ok = NSWorkspace.shared.open(app)
            reply(context, ["ok": ok])
        }
    }

    /// Erstes Bündel mit der Endung .app oberhalb dieser Erweiterung.
    private func containingApp() -> URL? {
        var url = Bundle.main.bundleURL
        while url.pathComponents.count > 1 {
            url.deleteLastPathComponent()
            if url.pathExtension == "app" {
                return url
            }
        }
        return nil
    }

    private func reply(_ context: NSExtensionContext, _ body: [String: Any]) {
        let response = NSExtensionItem()
        if #available(macOS 11.0, *) {
            response.userInfo = [SFExtensionMessageKey: body]
        } else {
            response.userInfo = ["message": body]
        }
        context.completeRequest(returningItems: [response], completionHandler: nil)
    }
}
