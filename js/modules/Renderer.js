(function () {
    'use strict';

    var alchemy = require('./Alchemy.js');

    /**
     * An appliction module to render all view components
     * to the screen
     *
     * @class
     * @name arena.modules.Renderer
     * @extends alchemy.core.MateriaPrima
     */
    alchemy.formula.add({
        name: 'arena.modules.Renderer',
        extend: 'alchemy.core.MateriaPrima',
        ingredients: [{
            key: 'mod',
            ptype: 'arena.modules.Prima'
        }],
        overrides: {
            /** @lends arena.modules.Renderer.prototype */

            update: function (params) {
                var views = this.entities.getComponent('view');
                if (views) {
                    views.each(this.updateView, this, [params]);
                }
            },

            updateView: function (view, id, params) {
                this.renderView(view);
                view.update(params);
            },

            renderView: function (view) {
                if (view.rendered && view.dirty !== true) {
                    // no further rendering required
                    return;
                }

                // get the target (parent) dom element
                var target = view.parent;
                if (alchemy.isString(target)) {
                    target = $(target)[0];
                    view.parent = target;
                }

                if (alchemy.isObject(target)) {
                    target.insertAdjacentHTML('beforeend', view.render([]).join(''));
                    view.setEl(target.children[target.children.length - 1]);
                    view.dirty = false;
                    view.rendered = true;
                    view.trigger('rendered');
                }
            }
        }
    });
}());
