on run argv
    set notesList to {}

    tell application "Notes"
        set theAccounts to every account
        repeat with anAccount in theAccounts
            set accountName to name of anAccount
            set theFolders to every folder of anAccount
            repeat with aFolder in theFolders
                set folderName to name of aFolder
                set theNotes to notes of aFolder

                repeat with theNote in theNotes
                    -- Get note data
                    set noteTitle to name of theNote
                    set noteContent to body of theNote
                    set noteId to id of theNote
                    set noteCreationDate to creation date of theNote
                    set noteModificationDate to modification date of theNote

                    -- Clean strings by removing problematic characters
                    set cleanTitle to my cleanForJson(noteTitle)
                    set cleanContent to my cleanForJson(noteContent)
                    set cleanFolder to my cleanForJson(folderName)
                    set cleanAccount to my cleanForJson(accountName)

                    -- Create JSON object
                    set noteData to "{"
                    set noteData to noteData & "\"title\":\"" & cleanTitle & "\","
                    set noteData to noteData & "\"content\":\"" & cleanContent & "\","
                    set noteData to noteData & "\"folder\":\"" & cleanFolder & "\","
                    set noteData to noteData & "\"account\":\"" & cleanAccount & "\","
                    set noteData to noteData & "\"id\":\"" & noteId & "\","
                    set noteData to noteData & "\"created\":\"" & ((noteCreationDate as text)) & "\","
                    set noteData to noteData & "\"modified\":\"" & ((noteModificationDate as text)) & "\""
                    set noteData to noteData & "}"

                    set end of notesList to noteData
                end repeat
            end repeat
        end repeat
    end tell

    return "[" & (my joinList(notesList, ",")) & "]"
end run

on joinList(theList, theDelimiter)
    set oldDelimiters to AppleScript's text item delimiters
    set AppleScript's text item delimiters to theDelimiter
    set theString to theList as string
    set AppleScript's text item delimiters to oldDelimiters
    return theString
end joinList

on cleanForJson(str)
    -- Replace these characters in this specific order
    set str to my replaceText(str, "\\", "\\\\") -- Escape backslashes first
    set str to my replaceText(str, "\"", "\\\"") -- Escape double quotes
    set str to my replaceText(str, "\n", "\\n") -- Replace newlines with \n
    set str to my replaceText(str, "\r", "\\r") -- Replace returns with \r
    set str to my replaceText(str, "\t", "\\t") -- Replace tabs with \t
    set str to my replaceText(str, "/", "\\/") -- Escape forward slashes
    return str
end cleanForJson

on replaceText(sourceText, searchString, replacementString)
    set AppleScript's text item delimiters to searchString
    set the textItems to every text item of sourceText
    set AppleScript's text item delimiters to replacementString
    set sourceText to textItems as string
    set AppleScript's text item delimiters to ""
    return sourceText
end replaceText
