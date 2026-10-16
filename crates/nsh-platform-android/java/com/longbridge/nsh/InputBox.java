package com.longbridge.nsh;

import android.annotation.SuppressLint;
import android.content.Context;
import android.graphics.drawable.ColorDrawable;
import android.os.Build;
import android.text.Editable;
import android.text.InputFilter;
import android.text.InputType;
import android.text.TextWatcher;
import android.util.TypedValue;
import android.view.Gravity;
import android.view.View;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputConnection;
import android.view.inputmethod.InputConnectionWrapper;
import android.view.inputmethod.InputMethodManager;
import android.widget.EditText;
import android.widget.FrameLayout;

/** `<input>`: an EditText inside a container that carries the decoration. */
@SuppressLint("ViewConstructor")
final class InputBox extends FrameLayout implements Node {
    final Mounter mounter;
    final int id;
    final Decor decor = new Decor();
    final EditText edit;
    /** Edits reported to the engine. */
    int count;
    /** The text as last reported or set. */
    private String known = "";
    private boolean multiline;
    /** Return unfocuses a single-line input. */
    private boolean blurOnSubmit = true;
    private boolean applying;
    private boolean configured;

    InputBox(Context context, Mounter mounter, int id) {
        super(context);
        this.mounter = mounter;
        this.id = id;
        setBackground(decor);
        edit = new EditText(context) {
            @Override
            public boolean onKeyUp(int code, android.view.KeyEvent event) {
                // The up of an Enter whose down submitted another field (focus
                // moved meanwhile) must not advance focus again.
                if (code == android.view.KeyEvent.KEYCODE_ENTER && !multiline) return true;
                return super.onKeyUp(code, event);
            }

            @Override
            public boolean onKeyDown(int code, android.view.KeyEvent event) {
                // Hardware keys, and keys the keyboard sends as key events.
                if (code == android.view.KeyEvent.KEYCODE_DEL) key("Backspace");
                else if (code == android.view.KeyEvent.KEYCODE_ENTER) key("Enter");
                else if (event.getUnicodeChar() != 0) key(String.valueOf((char) event.getUnicodeChar()));
                return super.onKeyDown(code, event);
            }

            @Override
            public InputConnection onCreateInputConnection(EditorInfo info) {
                InputConnection connection = super.onCreateInputConnection(info);
                return connection == null ? null : new KeyConnection(connection);
            }

            @Override
            protected void onSelectionChanged(int start, int end) {
                super.onSelectionChanged(start, end);
                if (mounter != null && mounter.wants(InputBox.this.id, Native.SELECTION_CHANGE)) {
                    mounter.eventMap(InputBox.this.id, Native.SELECTION_CHANGE, new String[] {"start", "end"},
                            new Object[] {(double) start, (double) end});
                }
            }
        };
        edit.setBackground(new ColorDrawable(0));
        edit.setPadding(0, 0, 0, 0);
        edit.setIncludeFontPadding(false);
        edit.addTextChangedListener(new TextWatcher() {
            @Override
            public void beforeTextChanged(CharSequence s, int start, int count, int after) {}

            @Override
            public void onTextChanged(CharSequence s, int start, int before, int count) {}

            @Override
            public void afterTextChanged(Editable s) {
                if (applying) return;
                // Reconfiguring (filters, input type) sets the same text
                // again: only real edits count.
                String text = s.toString();
                if (text.equals(known)) return;
                known = text;
                InputBox.this.count++;
                mounter.event(id, Native.CHANGE_TEXT, s.toString(), InputBox.this.count, 0, 0);
            }
        });
        edit.setOnEditorActionListener((v, action, event) -> {
            if (multiline && event != null) return false;
            // The keyboard's action key (a hardware Enter reported its key).
            if (event == null) key("Enter");
            // A hardware Enter reports its down and its up: act once. (The up
            // would otherwise reach the next field, focused meanwhile.)
            if (event != null && event.getAction() != android.view.KeyEvent.ACTION_DOWN) return true;
            mounter.event(id, Native.SUBMIT_EDITING, edit.getText().toString(), count, 0, 0);
            if (!multiline && blurOnSubmit) blur();
            return true;
        });
        edit.setOnFocusChangeListener((v, focused) -> {
            mounter.inputFocused(this, focused);
            mounter.reportState(id, Mounter.STATE_FOCUSED, focused);
            if (mounter.wants(id, focused ? Native.FOCUS : Native.BLUR)) {
                mounter.event(id, focused ? Native.FOCUS : Native.BLUR);
            }
        });
        addView(edit, new LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT));
    }

    void configure(float fontSizeDp, int weight, boolean italic, int color, String family, float letterSpacingDp,
                   int align, String placeholder, boolean hasPlaceholderColor, int placeholderColor,
                   boolean secure, int keyboard, int returnKey, int capitalize, boolean autoCorrect,
                   boolean multiline, int maxLength, boolean editable, boolean autoFocus,
                   boolean hasSelectionColor, int selectionColor, boolean blurOnSubmit) {
        this.multiline = multiline;
        this.blurOnSubmit = blurOnSubmit;
        edit.setTextSize(TypedValue.COMPLEX_UNIT_PX, fontSizeDp * mounter.density);
        edit.setTextColor(color);
        if (letterSpacingDp != 0) edit.setLetterSpacing(letterSpacingDp / fontSizeDp);
        int horizontal = align == 1 ? Gravity.CENTER_HORIZONTAL : align == 2 ? Gravity.END : Gravity.START;
        edit.setGravity(horizontal | (multiline ? Gravity.TOP : Gravity.CENTER_VERTICAL));
        edit.setHint(placeholder);
        if (hasPlaceholderColor) edit.setHintTextColor(placeholderColor);
        int type;
        switch (keyboard) {
            case 1: type = InputType.TYPE_CLASS_NUMBER; break;
            case 2: type = InputType.TYPE_CLASS_NUMBER | InputType.TYPE_NUMBER_FLAG_DECIMAL; break;
            // `numeric` (as React Native) and `numbers-and-punctuation`: a sign too.
            case 6:
            case 7:
                type = InputType.TYPE_CLASS_NUMBER | InputType.TYPE_NUMBER_FLAG_SIGNED
                        | InputType.TYPE_NUMBER_FLAG_DECIMAL;
                break;
            case 3: type = InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_VARIATION_EMAIL_ADDRESS; break;
            case 4: type = InputType.TYPE_CLASS_PHONE; break;
            case 5: type = InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_VARIATION_URI; break;
            default: type = InputType.TYPE_CLASS_TEXT;
        }
        if ((type & InputType.TYPE_MASK_CLASS) == InputType.TYPE_CLASS_TEXT) {
            switch (capitalize) {
                case 1: type |= InputType.TYPE_TEXT_FLAG_CAP_SENTENCES; break;
                case 2: type |= InputType.TYPE_TEXT_FLAG_CAP_WORDS; break;
                case 3: type |= InputType.TYPE_TEXT_FLAG_CAP_CHARACTERS; break;
                default: break;
            }
            type |= autoCorrect ? InputType.TYPE_TEXT_FLAG_AUTO_CORRECT : InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS;
            if (multiline) type |= InputType.TYPE_TEXT_FLAG_MULTI_LINE;
            if (secure) type = (type & ~InputType.TYPE_MASK_VARIATION) | InputType.TYPE_TEXT_VARIATION_PASSWORD;
        } else if (secure && (type & InputType.TYPE_MASK_CLASS) == InputType.TYPE_CLASS_NUMBER) {
            type |= InputType.TYPE_NUMBER_VARIATION_PASSWORD;
        }
        int selection = edit.getSelectionEnd();
        // Before the input type: setSingleLine replaces a password's
        // transformation (the text would show).
        edit.setSingleLine(!multiline);
        if (multiline) edit.setMaxLines(Integer.MAX_VALUE);
        edit.setInputType(type);
        // After the input type: password types reset the typeface to monospace.
        edit.setTypeface(TextLayouts.typeface(family, weight, italic));
        if (selection >= 0 && selection <= edit.length()) edit.setSelection(selection);
        int ime;
        switch (returnKey) {
            case 1: ime = EditorInfo.IME_ACTION_DONE; break;
            case 2: ime = EditorInfo.IME_ACTION_GO; break;
            case 3: ime = EditorInfo.IME_ACTION_NEXT; break;
            case 4: ime = EditorInfo.IME_ACTION_SEARCH; break;
            case 5: ime = EditorInfo.IME_ACTION_SEND; break;
            default: ime = multiline ? EditorInfo.IME_ACTION_NONE : EditorInfo.IME_ACTION_DONE;
        }
        edit.setImeOptions(ime);
        edit.setFilters(maxLength >= 0 ? new InputFilter[] {new InputFilter.LengthFilter(maxLength)} : new InputFilter[0]);
        edit.setEnabled(editable);
        if (hasSelectionColor && Build.VERSION.SDK_INT >= 29) {
            edit.setTextCursorDrawable(new ColorDrawable(selectionColor) {
                @Override
                public int getIntrinsicWidth() {
                    return Math.round(2 * mounter.density);
                }
            });
            edit.setHighlightColor((selectionColor & 0x00ffffff) | 0x55000000);
        } else if (!hasSelectionColor) {
            edit.setHighlightColor(0x5533b5e5);
        }
        boolean first = !configured;
        configured = true;
        if (first && autoFocus) post(this::focus);
    }

    /** `onKeyPress`. */
    private void key(String key) {
        if (mounter.wants(id, Native.KEY_PRESS)) {
            mounter.eventMap(id, Native.KEY_PRESS, new String[] {"key"}, new Object[] {key});
        }
    }

    /**
     * Reports the keys of soft keyboards, which edit through the input
     * connection rather than with key events.
     */
    private final class KeyConnection extends InputConnectionWrapper {
        /** The text being composed (a word the keyboard may still change). */
        private CharSequence composing;

        KeyConnection(InputConnection target) {
            super(target, true);
        }

        @Override
        public boolean setComposingText(CharSequence text, int position) {
            int before = composing == null ? 0 : composing.length();
            if (text.length() > before) key(text.subSequence(text.length() - 1, text.length()).toString());
            else if (text.length() < before) key("Backspace");
            composing = text;
            return super.setComposingText(text, position);
        }

        @Override
        public boolean finishComposingText() {
            composing = null;
            return super.finishComposingText();
        }

        @Override
        public boolean commitText(CharSequence text, int position) {
            String added = text.toString();
            // Committing the word being composed adds only what is new.
            if (composing != null && added.startsWith(composing.toString())) added = added.substring(composing.length());
            composing = null;
            if (added.equals("\n")) key("Enter");
            else if (!added.isEmpty()) key(added);
            return super.commitText(text, position);
        }

        @Override
        public boolean deleteSurroundingText(int before, int after) {
            if (before > 0 || after > 0) key("Backspace");
            return super.deleteSurroundingText(before, after);
        }
    }

    /** Selects {@code start..end} (clamped to the text). */
    void select(int start, int end) {
        int length = edit.length();
        edit.setSelection(Math.min(start, length), Math.min(Math.max(start, end), length));
    }

    /** Autofill: a hint ("" for none, null for the default). */
    void setAutofill(String hint) {
        if (Build.VERSION.SDK_INT < 26) return;
        if (hint == null) {
            edit.setImportantForAutofill(View.IMPORTANT_FOR_AUTOFILL_AUTO);
            edit.setAutofillHints((String[]) null);
        } else if (hint.isEmpty()) {
            edit.setImportantForAutofill(View.IMPORTANT_FOR_AUTOFILL_NO);
        } else {
            edit.setImportantForAutofill(View.IMPORTANT_FOR_AUTOFILL_YES);
            edit.setAutofillHints(hint);
        }
    }

    /** Sets the text from JS unless the user typed since JS saw `count`. */
    void setValue(String text, int count) {
        if (count != this.count || text.contentEquals(edit.getText())) return;
        applying = true;
        known = text;
        int selection = edit.getSelectionEnd();
        edit.setText(text);
        edit.setSelection(Math.min(Math.max(selection, 0), text.length()));
        applying = false;
    }

    void setContent(int left, int top, int right, int bottom) {
        // Padding of the container: the EditText fills the content box.
        setPadding(left, top, right, bottom);
    }

    void focus() {
        edit.requestFocus();
        InputMethodManager imm = (InputMethodManager) getContext().getSystemService(Context.INPUT_METHOD_SERVICE);
        imm.showSoftInput(edit, InputMethodManager.SHOW_IMPLICIT);
    }

    void blur() {
        if (!edit.hasFocus()) return;
        hideKeyboard();
        // To the (focusable) root: clearFocus() would hand focus to the first
        // focusable view, often another field.
        mounter.host.requestFocus();
    }

    private void hideKeyboard() {
        InputMethodManager imm = (InputMethodManager) getContext().getSystemService(Context.INPUT_METHOD_SERVICE);
        imm.hideSoftInputFromWindow(edit.getWindowToken(), 0);
    }

    @Override
    public int nodeId() {
        return id;
    }
}
