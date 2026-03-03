import Foundation
import Contacts

@Observable
final class ContactsSensor {
    private let store = CNContactStore()
    var isAuthorized = false

    struct ContactEvent {
        let type: String
        let source: String
        let timestamp: Date
        let data: [String: String]
    }

    func requestAuthorization() async -> Bool {
        do {
            let granted = try await store.requestAccess(for: .contacts)
            isAuthorized = granted
            return granted
        } catch {
            print("Contacts auth failed: \(error)")
            return false
        }
    }

    func fetchContacts() async -> [ContactEvent] {
        guard isAuthorized else { return [] }

        let keys: [CNKeyDescriptor] = [
            CNContactGivenNameKey as CNKeyDescriptor,
            CNContactFamilyNameKey as CNKeyDescriptor,
            CNContactOrganizationNameKey as CNKeyDescriptor,
        ]

        let request = CNContactFetchRequest(keysToFetch: keys)
        var events: [ContactEvent] = []

        do {
            try store.enumerateContacts(with: request) { contact, _ in
                let name = "\(contact.givenName) \(contact.familyName)".trimmingCharacters(in: .whitespaces)
                guard !name.isEmpty else { return }

                events.append(ContactEvent(
                    type: "social.contact",
                    source: "contacts",
                    timestamp: Date(),
                    data: [
                        "name": name,
                        "organization": contact.organizationName,
                    ]
                ))
            }
        } catch {
            print("Contact fetch failed: \(error)")
        }

        return events
    }

    func contactCount() async -> Int {
        let events = await fetchContacts()
        return events.count
    }
}
