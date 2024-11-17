(function () {
    'use strict';

    customElements.define('custom-tabs', class extends HTMLElement {
        constructor() {
            super();
            this._selected = null;

            // Create shadow DOM for the component.
            let shadowRoot = this.attachShadow({ mode: 'open' });
            shadowRoot.innerHTML = `
                <style>
                    :host {
                        display: flex;
                        flex-direction: column;
                        width: 100%;
                        border: 1px solid var(--mdc-theme-divider);
                        border-radius: 4px;
                    }

                    #tabs {
                        display: flex;
                        border-bottom: 1px solid var(--mdc-theme-divider);
                        background-color: var(--mdc-theme-primary);
                        overflow-x: auto;
                        position: relative;
                    }

                    #tabs ::slotted(*) {
                        color: var(--mdc-theme-text-primary);
                        padding: 12px 16px;
                        text-align: center;
                        text-overflow: ellipsis;
                        white-space: nowrap;
                        overflow: hidden;
                        cursor: pointer;
                        border-bottom: 2px solid transparent;
                        transition: border-bottom-color 0.3s, background-color 0.3s;
                        margin: 0;
                        font-size: 14px;
                        font-weight: bold;
                    }

                    #tabs ::slotted([tabindex="0"]), #tabs ::slotted(*:hover) {
                        color: var(--mdc-theme-primary);
                        background-color: var(--mdc-theme-background);
                        border-bottom-color: var(--mdc-theme-primary);
                    }

                    #tabsLine {
                        border-top: 1px solid var(--mdc-theme-divider);
                        margin-top: -1px;
                        position: absolute;
                        width: 100%;
                        z-index: 1;
                    }

                    #panels {
                        padding: 0px;
                    }

                    #panels ::slotted([aria-hidden="true"]) {
                        display: none;
                    }

                    pre {
                        margin: 0;
                    }

                    /* Responsive styles */
                    @media (max-width: 600px) {
                        #tabs {
                            flex-wrap: nowrap;
                        }

                        #tabs ::slotted(*) {
                            flex-grow: 1;
                            flex-shrink: 0;
                        }
                    }
                </style>
                <div id="tabs">
                    <slot id="tabsSlot" name="title"></slot>
                </div>
                <div id="panels">
                    <slot id="panelsSlot"></slot>
                </div>
            `;
        }

        get selected() {
            return this._selected;
        }

        set selected(idx) {
            this._selected = idx;
            this._selectTab(idx);
            this.setAttribute('selected', idx);
        }

        connectedCallback() {
            this.setAttribute('role', 'tablist');
            const tabsSlot = this.shadowRoot.querySelector('#tabsSlot');
            const panelsSlot = this.shadowRoot.querySelector('#panelsSlot');
            this.tabs = tabsSlot.assignedNodes({ flatten: true });
            this.panels = panelsSlot.assignedNodes({ flatten: true }).filter(el => el.nodeType === Node.ELEMENT_NODE);
            // Save refer to we can remove listeners later.
            this._boundOnTitleClick = this._onTitleClick.bind(this);
            this._boundOnSiblingCategoryChanged = this._onSiblingCategoryChanged.bind(this);
            tabsSlot.addEventListener('click', this._boundOnTitleClick);
            document.addEventListener('mdbook-category-changed', this._boundOnSiblingCategoryChanged);
            this.selected = this._findFirstSelectedTab() || this._findStoredSelectedTab() || 0;
        }

        disconnectedCallback() {
            const tabsSlot = this.shadowRoot.querySelector('#tabsSlot');
            tabsSlot.removeEventListener('click', this._boundOnTitleClick);
            document.removeEventListener('mdbook-category-changed', this._boundOnSiblingCategoryChanged);
        }

        _onTitleClick(e) {
            if (e.target.slot === 'title') {
                this.selected = this.tabs.indexOf(e.target);
                e.target.focus();
            }
        }

        _findFirstSelectedTab() {
            let selectedIdx;
            for (let [i, tab] of this.tabs.entries()) {
                tab.setAttribute('role', 'tab');
                if (tab.hasAttribute('selected')) {
                    selectedIdx = i;
                }
            }
            return selectedIdx;
        }

        _findStoredSelectedTab() {
            let selectedIdx;
            if (this.getAttribute("category")) {
                let selectedText;
                try {
                    selectedText = localStorage.getItem('mdbook-tabs-' + this.getAttribute("category"));
                } catch (e) {
                    console.error('Error accessing localStorage', e);
                }
                if (selectedText) {
                    for (let [i, tab] of this.tabs.entries()) {
                        if (tab.textContent === selectedText) {
                            selectedIdx = i;
                            break;
                        }
                    }
                }
            }
            return selectedIdx;
        }

        _selectTab(idx = null, propagate = true) {
            let category = this.getAttribute("category");
            for (let i = 0, tab; tab = this.tabs[i]; ++i) {
                let select = i === idx;
                tab.setAttribute('tabindex', select ? 0 : -1);
                tab.setAttribute('aria-selected', select);
                this.panels[i].setAttribute('aria-hidden', !select);
                if (select && category && tab.textContent) {
                    try {
                        localStorage.setItem('mdbook-tabs-' + category, tab.textContent);
                    } catch (e) {
                        console.error('Error accessing localStorage', e);
                    }
                }
            }
            if (propagate) {
                document.dispatchEvent(new CustomEvent(
                    'mdbook-category-changed',
                    { detail: { category: category, idx: idx }}
                ));
            }
        }

        _onSiblingCategoryChanged(e) {
            let category = this.getAttribute("category")
            if (category === e.detail.category) {
                this._selectTab(e.detail.idx, false);
                this.setAttribute('selected', e.detail.idx);
            }
        }
    });
})();
