/*global $*/
(function () {
    'use strict';

    var alchemy = require('./alchemy.js');

    /**
     * Description
     *
     * @class arena.view.Prima
     * @extends Oculus
     */
    alchemy.formula.add({
        name: 'arena.view.Prima',
        extend: 'alchemy.core.Oculus',
        overrides: {
            template: '<div id="<$=data.id$>" class="<$=data.cls$>"><$=data.items$></div>',

            init: function hocuspocus(_super) {
                return function () {
                    _super.call(this);

                    if (this.target) {
                        this.renderTo(this.target);
                        delete this.target;
                    }
                };
            },

            getData: function () {
                return {
                    id: this.id,
                    cls: this.cls
                };
            },

            render: function (ctxt) {
                ctxt.push(alchemy.render(this.template, this.getData()));
                return ctxt;
            },

            renderTo: function (target) {
                if (alchemy.isString(target)) {
                    target = $(target)[0];
                }
                if (alchemy.isObject(target)) {
                    target.insertAdjacentHTML('beforeend', this.render([]).join(''));
                    this.setEl(target.children[target.children.length - 1]);
                    this.trigger('rendered');
                }
            },

            /**
             * Adds new style class(es) to the view element
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
                if (alchemy.isString(newCls)) {
                    newCls = newCls.trim();
                    if (newCls.indexOf(' ') > 0) {
                        newCls = newCls.split(' ');
                    }
                }

                if (alchemy.isArray(newCls)) {
                    alchemy.each(newCls, this.addClass, this);
                } else {
                    if (this.cls) {
                        if (this.cls.indexOf(newCls) < 0) {
                            this.cls += ' ' + newCls;
                        }
                    } else {
                        this.cls = newCls;
                    }
                }
                return this;
            },


            setEl: function (el) {
                if (el) {
                    this.el = el;
                    this.$el = $(el);
                }
                return this.el;
            }
        }
    });
}());

