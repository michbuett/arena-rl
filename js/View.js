/*global $*/
(function () {
    'use strict';

    var alchemy = require('./alchemy.js');

    /**
     * Description
     *
     * @class arena.View
     * @extends Oculus
     */
    alchemy.formula.add({
        name: 'arena.View',
        extend: 'Oculus',
        overrides: {
            template: '<div id="<$=data.id$>" class="<$=data.cls$>"><$=data.items$></div>',

            init: function () {
                _super.call(this);

                if (this.target) {
                    this.renderTo(this.target);
                    delete this.target;
                }
            },

            getData: (function () {
                function renderItem(item, key, ctxt) {
                    item.render(ctxt);
                }
                return function () {
                    var itemCtxt;
                    if (this.items) {
                        itemCtxt = [];
                        alchemy.each(this.items, renderItem, this, [itemCtxt]);
                    }
                    return {
                        id: this.id,
                        cls: this.cls,
                        items: itemCtxt && itemCtxt.join('')
                    };
                };
            }()),

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

