Alchemy.ns('Alchemy.v');

/**
 * @class Alchemy.v.DomElement
 * @extends Alchemy.v.BasicElement
 *
 * The basic type representing a simple HTML element
 */
(function () {
    var DOM_EVENTS = ['click', 'mousedown', 'mouseup'].concat(Alchemy.kb.events);
    var DomElement = {
        /**
        * @cfg {String} tag
        * the element tag; defaults to <code>div</code>
        */
        tag: 'div',

        /**
        * @cfg {String} cls
        * the style class; multiple classes should be separated with spaces
        */
        cls: '',

        /**
        * @cfg {String} html
        * the inner html content
        */
        html: '',

        /**
        * @cfg {Object} style
        * the style attributes
        */
        style: undefined,

        /**
        * @cfg {HTMLElement} target
        * the HTML element to which this element should be rendered
        */
        target: undefined,

        /**
        * the reference to the actual DOM element or a selector to the element;
        * read-only
        *
        * @property dom
        * @type HTMLElement/String
        */
        dom: undefined,

        /**
        * the constructor method
        * @constructor
        */
        constructor: function (cfg) {
            this.domEventListeners = {};

            if (cfg && cfg.cls) {
                this.addClass(cfg.cls);
                delete cfg.cls;
            }

            _super.call(this, cfg);

            if (Alchemy.isNumber(this.x)) {
                this.setStyle({position: 'absolute', left: this.x + 'px'});
            }
            if (Alchemy.isNumber(this.y)) {
                this.setStyle({position: 'absolute', top: this.y + 'px'});
            }
            if (Alchemy.isNumber(this.width)) {
                this.setStyle({width: this.width + 'px'});
            }
            if (Alchemy.isNumber(this.height)) {
                this.setStyle({height: this.height + 'px'});
            }

            if (typeof this.dom === 'string') {
                // try to link to a dom element if an id is given
                var el = Alchemy.v.DomHelper.get(this.dom);
                this.dom = undefined;
                if (el) {
                    this.linkDom(el);
                }
            } else if (this.target) {
                this.renderTo(this.target);
                delete this.target;
            }
        },

        /**
        * @param {HTMLElement} parentDom
        *      the parent DOM container element
        *
        * @param {String} position
        *      the position at where to add the element (defaults to
        *      <code>'beforeend'</code>
        */
        renderTo: function (parentDom, position) {
            // add element to parent
            position = position || 'beforeend';
            parentDom.insertAdjacentHTML(position, this.render([]).join(''));
            this.linkDom();
        },

        /**
         * links the alchemy.js element to a DOM element
         * @protected
         */
        linkDom: function (element) {
            var oldDom = this.dom;
            // link dom element
            var newDom = this.dom = element || Alchemy.v.DomHelper.get('#' + this.id);
            if (newDom) {
                if (!Alchemy.isNumber(this.x)) {
                    this.x = newDom.offsetLeft;
                }
                if (!Alchemy.isNumber(this.y)) {
                    this.y = newDom.offsetTop;
                }
                if (!Alchemy.isNumber(this.width)) {
                    this.width = newDom.offsetWidth;
                }
                if (!Alchemy.isNumber(this.height)) {
                    this.height = newDom.offsetHeight;
                }
            }
            // add cached dom listeners to new dom element and clean
            // previous dom element to avoid memory leaks
            var domListerners = this.domEventListeners;
            for (var event in domListerners) {
                if (domListerners.hasOwnProperty(event)) {
                    var l = domListerners[event];
                    if (oldDom) {
                        oldDom.removeEventListener(event, l);
                    }
                    if (newDom) {
                        newDom.addEventListener(event, l);
                    }
                }
            }

        },

        isDomEvent: function (eventType) {
            return (DOM_EVENTS.indexOf(eventType) >= 0);
        },

        addListener: function (event, handler, scope) {
            _super.call(this, event, handler, scope);

            // apply special handling of dom events
            if (this.isDomEvent(event) && !this.domEventListeners[event]) {
                var domEventHandler = (function (e) {
                    this.fireEvent(event, {
                        src: this,
                        event: e
                    });
                }).bind(this);

                this.domEventListeners[event] = domEventHandler;

                if (this.dom) {
                    this.dom.addEventListener(event, domEventHandler);
                }
            }
        },

        removeListener: function (event, handler, scope) {
            _super.call(this, event, handler, scope);

            if (this.isDomEvent(event)) {
                if (!this.hasListener(event) && this.domEventListeners[event]) {
                    var listener = this.domEventListeners[event];
                    delete this.domEventListeners[event];

                    if (this.dom) {
                        this.dom.removeEventListener(event, listener);
                    }
                }
            }
        },

        /**
         * this function renders the element into the given parent container
         *
         * @return String
         *      the html output
         */
        render: function (ctxt) {
            // add basic attributes
            ctxt.push('<' + this.tag + ' id=' + this.id + ' class="' + this.cls + '"');
            // add styles (if nessecary)
            if (this.style) {
                ctxt.push(' style="')
                for (var s in this.style) {
                    ctxt.push(s + ':' + this.style[s] + ';');
                }
                ctxt.push('"');
            }
            // add further attributes
            this.renderAttributes(ctxt);
            ctxt.push('>');
            // add content
            this.renderContent(ctxt);
            // close tag
            ctxt.push('</' + this.tag + '>');

            return ctxt;
        },

        renderAttributes: function (ctxt) {
            return ctxt;
        },

        /**
        * this function renders the element's content
        *
        * @return String
        *      the html output of the element's content
        */
        renderContent: function (ctxt) {
            if (this.html) {
                ctxt.push(this.html);
            }
            return ctxt;
        },

        /**
        * updates the content of the element
        *
        * @param {String} html
        *      the new inner html
        */
        setContent: function (html) {
            this.html = html;
            this.updateUI();
        },

        updateUI: function () {
            /*
            if (this.dom) {
                this.dom.innerHTML = this.renderContent([]).join('');
            }
            */
        },

        /**
        * set the elements style attributes
        *
        * @param {Object} newStyle
        *      the new/update style value
        */
        setStyle: function (newStyle) {
            this.style = this.style || [];
            for (var key in newStyle) {
                if (newStyle.hasOwnProperty(key)) {
                    var newVal = newStyle[key];
                    if (this.style[key] !== newVal) {
                        this.style[key] = newVal;
                        if (this.dom) {
                            // this second version seems be a little bit more effective
                            //this.dom.style.setProperty(key, newVal);
                            this.dom.style[key] = newVal;
                        }
                    }
                }
            }
        },

        /**
         * adds new style class(es) to the element
         *
         * @param {String/Array} newCls
         *      the style class(es) to add
         *
         * @example
         *      el.addClass('foo');
         *      el.addClass('foo bar baz');
         *      el.addClass(['foo', 'bar', 'baz']);
         */
        addClass: function (newCls) {
            if (Alchemy.isString(newCls)) {
                newCls = newCls.trim();
                if (newCls.indexOf(' ') > 0) {
                    newCls = newCls.split(' ');
                }
            }
            if (Array.isArray(newCls)) {
                newCls.forEach(this.addClass, this);
            } else {
                if (this.cls) {
                    if (this.cls.indexOf(newCls) < 0) {
                        this.cls += ' ' + newCls;
                    }
                } else {
                    this.cls = newCls;
                }
            }
        },

        /**
        * moves the element to given position
        *
        * @param {Number} x
        *      the new x-coordinate
        *
        * @param {Number} y
        *      the new y-coordinate
        */
        moveTo: function (x, y) {
            //console.log('move to x=' + x + ', y=' + y);
            this.setStyle({
                position: 'absolute',
                left: x + 'px',
                top: y + 'px'
            });
            _super.call(this, x, y);
        },

        /**
        * changes the width of the current element;
        * overrides superclass to adapt the current style
        *
        * @param {Number} width
        *      the new width
        *
        * @return {Boolean}
        *      <code>true</code> iff the width has been changed
        */
        setWidth: function (width) {
            var changed = _super.call(this, width);
            if (changed) {
                this.setStyle({
                    width: width + 'px'
                });
            }
            return changed;
        },

        /**
        * changes the height of the current element;
        * overrides superclass to adapt the current style
        *
        * @param {Number} height
        *      the new height
        *
        * @return {Boolean}
        *      <code>true</code> iff the height has been changed
        */
        setHeight: function (height) {
            var changed = _super.call(this, height);
            if (changed) {
                this.setStyle({
                    height: height + 'px'
                });
            }
            return changed;
        },

        /**
         * masks the element
         *
         * @param {Object} cfg
         *      the mask element configuration
         *
         * @return {Object}
         *      the generated mask element
         */
        mask: function (cfg) {
            if (!this.isMasked()) {
                cfg = cfg || {};
                cfg = Alchemy.mix(cfg, {
                    x: this.getX() || 0,
                    y: this.getY() || 0,
                    width: this.getWidth(),
                    height: this.getHeight(),
                    type: cfg.type || Alchemy.v.LoadMask,
                    target: cfg.target || Alchemy.v.DomHelper.getBody()
                });
                this.maskEl = cfg.type.create(cfg);
            }
            return this.maskEl;
        },

        /**
         * removes the mask if the element is currently masked
         */
        unmask: function () {
            if (this.isMasked()) {
                this.maskEl.dispose();
                delete this.maskEl;
            }
        },

        /**
         * checks if the element is currently masked
         *
         * @return {Boolean}
         *      <code>true</code> if and only if the element is currently masked
         */
        isMasked: function () {
            return Alchemy.isObject(this.maskEl);
        },

        dispose: function () {
            _super.call(this);
            if (this.dom) {
                Alchemy.v.DomHelper.removeNode(this.dom);
                this.dom = undefined;
            }
        }
    };

    Alchemy.v.DomElement = Alchemy.brew({
        extend: Alchemy.v.BasicElement
    }, DomElement);
})();
